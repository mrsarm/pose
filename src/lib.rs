use std::collections::BTreeMap;
use std::path::Path;
use serde_yaml::{Mapping, Value, Error};

pub struct ComposeYaml {
    map: BTreeMap<String, Value>
}

impl ComposeYaml {
    pub fn new(yaml: &str) -> Result<ComposeYaml, Error> {
        let map = serde_yaml::from_str(&yaml)?;
        Ok(ComposeYaml { map })
    }

    pub fn get_root_element(&self, element_name: &str) -> Option<&Mapping> {
        let value = self.map.get(element_name);
        value.map(|v| v.as_mapping()).unwrap_or_default()
    }

    pub fn get_root_element_names(&self, element_name: &str) -> Vec<&str> {
        let elements = self.get_root_element(element_name);
        match elements {
            Some(s) => s.keys().map(|k| k.as_str().unwrap()).collect::<Vec<_>>(),
            None => Vec::default()
        }
    }
}

pub fn get_compose_filename(filename: &Option<String>) -> Result<String, &str> {
    let name = match filename {
        Some(name) => name,
        None =>
            if Path::new("compose.yaml").exists() {
                "compose.yaml"
            } else if Path::new("compose.yml").exists() {
                "compose.yml"
            } else if Path::new("docker-compose.yaml").exists() {
                "docker-compose.yaml"
            } else {
                "docker-compose.yml"
            }
    };
    if Path::new(&name).exists() {
        Ok(String::from(name))
    } else {
        Err("No such file")
    }
}
