use colored::*;
use regex::Regex;
use serde_yaml::{to_string, Error, Mapping, Value};
use std::collections::BTreeMap;
use std::path::Path;

lazy_static! {
    static ref EMPTY_MAP: Mapping = Mapping::default();
    static ref ENV_NAME_REGEX: Regex = Regex::new(r"^\w+$").unwrap();
}

pub struct ComposeYaml {
    map: BTreeMap<String, Value>,
}

impl ComposeYaml {
    pub fn new(yaml: &str) -> Result<ComposeYaml, Error> {
        let map = serde_yaml::from_str(yaml)?;
        Ok(ComposeYaml { map })
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

    pub fn get_images(&self) -> Option<Vec<&str>> {
        let services = self.get_services()?;
        let mut images = services
            .values()
            .flat_map(|v| v.as_mapping())
            .flat_map(|s| s.get("image"))
            .flat_map(|p| p.as_str())
            .collect::<Vec<_>>();
        images.sort();
        images.dedup();
        Some(images)
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
                        if val.contains(' ') {
                            if val.contains('"') {
                                format!("{env}='{}'", val.trim_end())
                            } else {
                                format!("{env}=\"{}\"", val.trim_end())
                            }
                        } else {
                            format!("{env}={}", val.trim_end())
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
static COMPOSE_PATHS: [&str; 8] = [
    "compose.yaml",
    "compose.yml",
    "docker-compose.yaml",
    "docker-compose.yml",
    "docker/compose.yaml",
    "docker/compose.yml",
    "docker/docker-compose.yaml",
    "docker/docker-compose.yml",
];

pub fn get_compose_filename(filename: &Option<String>) -> Result<String, String> {
    match filename {
        Some(name) => {
            if Path::new(&name).exists() {
                Ok(String::from(name))
            } else {
                Err(format!(
                    "{}: No such file or directory: '{}'",
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
                    "{}: Can't find a suitable configuration file in this directory.\n\
                    Are you in the right directory?\n\n\
                    Supported filenames: {}",
                    "ERROR".red(),
                    COMPOSE_PATHS.into_iter().collect::<Vec<&str>>().join(", ")
                )),
                1 => Ok(files.map(String::from).next().unwrap()),
                _ => {
                    let filenames = files.into_iter().collect::<Vec<&str>>();
                    let filename = filenames.first().map(|s| s.to_string()).unwrap();
                    eprintln!(
                        "{}: Found multiple config files with supported names: {}\n\
                        {}: Using {}",
                        "WARN".yellow(),
                        filenames.join(", "),
                        "WARN".yellow(),
                        filename
                    );
                    Ok(filename)
                }
            }
        }
    }
}
