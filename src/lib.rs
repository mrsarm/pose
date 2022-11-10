use std::collections::BTreeMap;
use std::path::Path;

pub fn get_compose_map(yaml: &str) -> Result<BTreeMap<String, serde_yaml::Value>, serde_yaml::Error> {
    let map: BTreeMap<String, serde_yaml::Value> = serde_yaml::from_str(&yaml)?;
    Ok(map)
}

pub fn get_compose_file(args: &Vec<String>) -> &str {
    let filename = if args.len() <= 1 {
        if Path::new("docker-compose.yaml").exists() {
            "docker-compose.yaml"
        } else {
            "docker-compose.yml"
        }
    } else {
        &args[1]
    };
    filename
}
