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
