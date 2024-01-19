use docker_pose::ComposeYaml;
use pretty_assertions::assert_eq;
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

#[test]
fn get_service_envs() -> Result<(), Error> {
    let yaml = r#"
services:
  app: the-app
  app1:
    image: some-image
    ports:
      - 8000:8000
    environment:
      - PORT=8000
      - KAFKA_BROKERS=kafka:9092
      - TITLE="App 1"
      - EMPTY=
      - UNDEFINED
      - UNDEFINED_TOO
    "#;
    let compose = ComposeYaml::new(&yaml)?;
    let app1 = compose.get_service("app1").expect("app1 not found");
    let envs = compose.get_service_envs(&app1);
    assert_eq!(
        envs.unwrap_or(Vec::default()),
        vec![
            "PORT=8000",
            "KAFKA_BROKERS=kafka:9092",
            "TITLE=\"App 1\"",
            "EMPTY=",
            "UNDEFINED=",
            "UNDEFINED_TOO=",
        ]
    );
    Ok(())
}

#[test]
fn get_services() -> Result<(), Error> {
    let yaml = "
services:
  app: the-app
  app-1:
    image: some-image
    ports:
      - 8000:8000
    ";
    let compose = ComposeYaml::new(&yaml)?;
    let app = compose.get_service("app");
    let app1 = compose.get_service("app-1");
    let not_exist = compose.get_service("does-not-exist");
    assert!(app.is_none()); // Because it's malformed
    assert!(app1.is_some());
    assert!(not_exist.is_none());
    Ok(())
}

#[test]
fn get_service_envs_empty() -> Result<(), Error> {
    let yaml = "
services:
  app: the-app
  app1:
    image: some-image
    ports:
      - 8000:8000
    ";
    let compose = ComposeYaml::new(&yaml)?;
    let app1 = compose.get_service("app1").expect("app1 not found");
    let envs = compose.get_service_envs(&app1);
    assert!(envs.is_none());
    Ok(())
}

#[test]
fn get_service_envs_with_map_notation() -> Result<(), Error> {
    let yaml = r#"
services:
  app: the-app
  app1:
    image: some-image
    ports:
      - 8000:8000
    environment:
      PORT: 8000
      KAFKA_BROKERS: "kafka:9092"
      TITLE: "App 1"
  app2:
    image: another-image:2.0
    ports:
      - 9000:9000
    "#;
    let compose = ComposeYaml::new(&yaml)?;
    let app1 = compose.get_service("app1").expect("app1 not found");
    let envs = compose.get_service_envs(&app1);
    assert_eq!(
        envs.unwrap_or(Vec::default()),
        vec!["PORT=8000", "KAFKA_BROKERS=kafka:9092", "TITLE=\"App 1\"",]
    );
    Ok(())
}
