use std::collections::BTreeMap;
use serde_yaml::{Mapping, Value};

pub fn get_services(map: &BTreeMap<String, Value>) -> Option<&Mapping> {
    let value = map.get("services");
    value.map(|v| v.as_mapping()).unwrap_or_default()
}

pub fn get_services_names(map: &BTreeMap<String, Value>) -> Vec<&str> {
    let services = get_services(&map);
    match services {
        Some(s) => s.keys().map(|k| k.as_str().unwrap()).collect::<Vec<_>>(),
        None => Vec::default()
    }
}
