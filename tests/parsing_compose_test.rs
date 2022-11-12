use pose::ComposeYaml;
use serde_yaml::Error;

#[test]
fn get_services_list() -> Result<(), Error> {
    let yaml = "
services:
  app: the-app
  app1:
    image: some-image
    ports:
      - 8000:8000
  app2:
    image: another-image:2.0
    ports:
      - 9000:9000
    depends_on:
      - app1

volumes:
  - no-body-cares
    ";
    let compose = ComposeYaml::new(&yaml)?;
    let services_map = compose.get_root_element("services").unwrap();
    let services_names = compose.get_root_element_names("services");
    assert_eq!(services_map.len(), services_names.len());
    assert_eq!(services_names, vec!["app", "app1", "app2"]);
    Ok(())
}

#[test]
fn get_services_empty_list() -> Result<(), Error> {
    let yaml = "
services: []
volumes:
  - no-body-cares
    ";
    let compose = ComposeYaml::new(&yaml)?;
    assert!(compose.get_root_element("services").is_none());
    assert!(compose.get_root_element_names("services").is_empty());
    Ok(())
}

#[test]
fn get_services_no_list() -> Result<(), Error> {
    let yaml = "
volumes:
  - no-body-cares
    ";
    let compose = ComposeYaml::new(&yaml)?;
    assert!(compose.get_root_element("services").is_none());
    assert!(compose.get_root_element_names("services").is_empty());
    Ok(())
}
