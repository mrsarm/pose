use crate::verbose::Verbosity;
use crate::DockerCommand;
use colored::*;
use regex::Regex;
use serde_yaml::{to_string, Error, Mapping, Value};
use std::collections::BTreeMap;
use std::path::Path;
use std::process;

lazy_static! {
    static ref EMPTY_MAP: Mapping = Mapping::default();
    static ref ENV_NAME_REGEX: Regex = Regex::new(r"^\w+$").unwrap();
    static ref QUOTED_NUM_REGEX: Regex = Regex::new(r"^'[0-9]+'$").unwrap();
}

pub struct ComposeYaml {
    map: BTreeMap<String, Value>,
}

pub struct RemoteTag {
    /// replace tag with remote tag if exists
    pub remote_tag: String,
    /// don't replace with remote tag unless this regex match the image name / tag
    pub remote_tag_filter: Option<Regex>,
    /// docker may require to be logged-in to fetch some images info
    pub ignore_unauthorized: bool,
    /// verbosity used when fetching remote images info
    pub verbosity: Verbosity,
}

impl ComposeYaml {
    pub fn new(yaml: &str) -> Result<ComposeYaml, Error> {
        let map = serde_yaml::from_str(yaml)?;
        Ok(ComposeYaml { map })
    }

    pub fn to_string(&self) -> Result<String, Error> {
        let yaml_string = to_string(&self.map)?;
        Ok(yaml_string)
    }

    pub fn get_root_element(&self, element_name: &str) -> Option<&Mapping> {
        let value = self.map.get(element_name);
        value.map(|v| v.as_mapping()).unwrap_or_default()
    }

    pub fn get_root_element_names(&self, element_name: &str) -> Vec<&str> {
        let elements = self.get_root_element(element_name).unwrap_or(&EMPTY_MAP);
        elements
            .keys()
            .map(|k| k.as_str().unwrap())
            .collect::<Vec<_>>()
    }

    pub fn get_services(&self) -> Option<&Mapping> {
        self.get_root_element("services")
    }

    pub fn get_profiles_names(&self) -> Option<Vec<&str>> {
        let services = self.get_services()?;
        let mut profiles = services
            .values()
            .flat_map(|v| v.as_mapping())
            .flat_map(|s| s.get("profiles"))
            .flat_map(|p| p.as_sequence())
            .flat_map(|seq| seq.iter())
            .flat_map(|e| e.as_str())
            .collect::<Vec<_>>();
        profiles.sort();
        profiles.dedup();
        Some(profiles)
    }

    pub fn get_images(
        &self,
        filter_by_tag: Option<&str>,
        remote_tag: Option<&RemoteTag>,
    ) -> Option<Vec<String>> {
        let services = self.get_services()?;
        let mut images = services
            .values()
            .flat_map(|v| v.as_mapping())
            .flat_map(|s| s.get("image"))
            .flat_map(|p| p.as_str())
            .filter(|image| match filter_by_tag {
                None => true,
                Some(tag) => {
                    let image_parts = image.split(':').collect::<Vec<_>>();
                    let image_tag = if image_parts.len() > 1 {
                        *image_parts.get(1).unwrap()
                    } else {
                        "latest"
                    };
                    tag == image_tag
                }
            })
            .collect::<Vec<_>>();
        images.sort();
        images.dedup();
        if let Some(remote) = remote_tag {
            let mut updated_images: Vec<String> = Vec::new();
            let command = DockerCommand::new(remote.verbosity.clone());
            for image in &images {
                if remote
                    .remote_tag_filter
                    .as_ref()
                    .map(|r| r.is_match(image))
                    .unwrap_or(true)
                {
                    let image_parts = image.split(':').collect::<Vec<_>>();
                    let image_name = *image_parts.first().unwrap();
                    let remote_image = format!("{}:{}", image_name, remote.remote_tag);
                    let inspect_output = command
                        .get_manifest_inspect(&remote_image)
                        .unwrap_or_else(|e| {
                            eprintln!(
                                "{}: fetching image manifest for {}: {}",
                                "ERROR".red(),
                                remote_image,
                                e
                            );
                            process::exit(151);
                        });
                    if inspect_output.status.success() {
                        if matches!(remote.verbosity, Verbosity::Verbose) {
                            eprintln!(
                                "{}: remote image {} found",
                                "DEBUG".green(),
                                remote_image.yellow()
                            );
                        }
                        updated_images.push(remote_image);
                    } else {
                        let exit_code = command.exit_code(&inspect_output);
                        let stderr = String::from_utf8(inspect_output.stderr).unwrap();
                        if stderr.contains("no such manifest")
                            || (remote.ignore_unauthorized && stderr.contains("unauthorized:"))
                        {
                            updated_images.push(image.to_string());
                        } else {
                            eprintln!(
                                "{}: fetching image manifest for {}: {}",
                                "ERROR".red(),
                                remote_image,
                                stderr
                            );
                            process::exit(exit_code);
                        }
                    }
                } else {
                    updated_images.push(image.to_string());
                }
            }
            return Some(updated_images);
        }
        Some(images.iter().map(|i| i.to_string()).collect::<Vec<_>>())
    }

    pub fn get_service(&self, service_name: &str) -> Option<&Mapping> {
        let services = self.get_services()?;
        let service = services.get(service_name);
        service.map(|v| v.as_mapping()).unwrap_or_default()
    }

    pub fn get_service_envs(&self, service: &Mapping) -> Option<Vec<String>> {
        let envs = service.get("environment")?;
        match envs.as_sequence() {
            Some(seq) => Some(
                seq.iter()
                    .map(|v| {
                        let val = v.as_str().unwrap_or("");
                        if ENV_NAME_REGEX.captures(val).is_some() {
                            // Env variable without a value or "=" at the end
                            format!("{val}=")
                        } else {
                            String::from(val)
                        }
                    })
                    .collect::<Vec<_>>(),
            ),
            None => Some(
                envs.as_mapping()
                    .unwrap_or(&EMPTY_MAP)
                    .into_iter()
                    .map(|(k, v)| {
                        let env = k.as_str().unwrap_or("".as_ref());
                        let val = to_string(v).unwrap_or("".to_string());
                        let val = val.trim_end();
                        if val.contains(' ') {
                            if val.contains('"') {
                                format!("{env}='{val}'")
                            } else {
                                format!("{env}=\"{val}\"")
                            }
                        } else if QUOTED_NUM_REGEX.captures(val).is_some() {
                            // remove unnecessary quotes
                            let val = &val[1..val.len() - 1];
                            format!("{env}={val}")
                        } else {
                            format!("{env}={val}")
                        }
                    })
                    .collect::<Vec<_>>(),
            ),
        }
    }

    pub fn get_service_depends_on(&self, service: &Mapping) -> Option<Vec<String>> {
        let depends = service.get("depends_on")?;
        match depends.as_sequence() {
            Some(seq) => Some(
                seq.iter()
                    .map(|el| el.as_str().unwrap_or(""))
                    .filter(|o| !o.is_empty())
                    .map(String::from)
                    .collect::<Vec<_>>(),
            ),
            None => Some(
                depends
                    .as_mapping()
                    .unwrap_or(&EMPTY_MAP)
                    .keys()
                    .map(|k| k.as_str().unwrap_or(""))
                    .filter(|o| !o.is_empty())
                    .map(String::from)
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

// where to look for the compose file when the user
// don't provide a path
static COMPOSE_PATHS: [&str; 4] = [
    "compose.yaml",
    "compose.yml",
    "docker-compose.yaml",
    "docker-compose.yml",
];

pub fn get_compose_filename(
    filename: Option<&str>,
    verbosity: Verbosity,
) -> Result<String, String> {
    match filename {
        Some(name) => {
            if Path::new(&name).exists() {
                Ok(String::from(name))
            } else {
                Err(format!(
                    "{}: {}: no such file or directory",
                    "ERROR".red(),
                    name
                ))
            }
        }
        None => {
            let files = COMPOSE_PATHS.into_iter().filter(|f| Path::new(f).exists());
            let files_count = files.clone().count();
            match files_count {
                0 => Err(format!(
                    "Can't find a suitable configuration file in this directory.\n\
                    Are you in the right directory?\n\n\
                    Supported filenames: {}",
                    COMPOSE_PATHS.into_iter().collect::<Vec<&str>>().join(", ")
                )),
                1 => {
                    let filename_0 = files.map(String::from).next().unwrap();
                    if matches!(verbosity, Verbosity::Verbose) {
                        eprintln!("{}: Filename not provided", "DEBUG".green());
                        eprintln!("{}: Using {}", "DEBUG".green(), filename_0);
                    }
                    Ok(filename_0)
                }
                _ => {
                    let filenames = files.into_iter().collect::<Vec<&str>>();
                    let filename = filenames.first().map(|s| s.to_string()).unwrap();
                    if !matches!(verbosity, Verbosity::Quiet) {
                        eprintln!(
                            "{}: Found multiple config files with supported names: {}\n\
                            {}: Using {}",
                            "WARN".yellow(),
                            filenames.join(", "),
                            "WARN".yellow(),
                            filename
                        );
                    }
                    Ok(filename)
                }
            }
        }
    }
}
