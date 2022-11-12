use std::collections::BTreeMap;
use serde_yaml::{Mapping, Value, Error};

pub struct ComposeYaml {
    map: BTreeMap<String, Value>
}

impl ComposeYaml {
    pub fn new(yaml: &str) -> Result<ComposeYaml, Error> {
        let map = serde_yaml::from_str(&yaml)?;
        Ok(ComposeYaml { map })
    }

    pub fn get_services(&self) -> Option<&Mapping> {
        let value = self.map.get("services");
        value.map(|v| v.as_mapping()).unwrap_or_default()
    }

    pub fn get_services_names(&self) -> Vec<&str> {
        let services = self.get_services();
        match services {
            Some(s) => s.keys().map(|k| k.as_str().unwrap()).collect::<Vec<_>>(),
            None => Vec::default()
        }
    }
}
