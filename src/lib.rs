#[macro_use]
extern crate lazy_static;

use colored::*;
use serde_yaml::{to_string, Error, Mapping, Value};
use std::collections::BTreeMap;
use std::path::Path;

lazy_static! {
    static ref EMPTY_MAP: Mapping = Mapping::default();
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

    pub fn get_service_envs(&self, service_name: &str) -> Option<Vec<String>> {
        let services = self.get_root_element("services")?;
        let service = services.get(service_name)?;
        let envs = service.get("environment")?;
        match envs.as_sequence() {
            None => Some(
                envs.as_mapping()
                    .unwrap_or(&EMPTY_MAP)
                    .into_iter()
                    .map(|(k, v)| {
                        let env = k.as_str().unwrap_or("".as_ref());
                        let val = to_string(v).unwrap_or("".to_string());
                        format!("{}={}", env, val.trim_end())
                    })
                    .collect::<Vec<_>>(),
            ),
            Some(seq) => Some(
                seq.iter()
                    .map(|v| String::from(v.as_str().unwrap_or("")))
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
        None => COMPOSE_PATHS
            .into_iter()
            .filter(|f| Path::new(f).exists())
            .map(String::from)
            .next()
            .ok_or(format!(
                "{}: Can't find a suitable configuration file in this directory.\n\
                 Are you in the right directory?\n\n\
                 Supported filenames: {}",
                "ERROR".red(),
                COMPOSE_PATHS.into_iter().collect::<Vec<&str>>().join(", ")
            )),
    }
}
