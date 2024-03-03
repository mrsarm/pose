use docker_pose::ComposeYaml;
use pretty_assertions::assert_eq;
use serde_yaml::Error;

#[test]
fn get_services_list() -> Result<(), Error> {
    let yaml = "
services:
  app: the-app
  # comments should not ...
  app1:
    # ... affects results
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
      - DESC_1='App 1 is the "Best"'
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
            "DESC_1='App 1 is the \"Best\"'",
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

#[test]
fn get_service_envs_with_map_notation_and_quoted() -> Result<(), Error> {
    let yaml = r#"
services:
  app1:
    image: some-image
    environment:
      PORT: 8000
      KAFKA_PORT: "9092"
      DB_PORT: '5432'
      TITLE: "App 1"
      DESC: 'Desc 1'
    "#;
    let compose = ComposeYaml::new(&yaml)?;
    let app1 = compose.get_service("app1").expect("app1 not found");
    let envs = compose.get_service_envs(&app1);
    assert_eq!(
        envs.unwrap_or(Vec::default()),
        vec![
            "PORT=8000",
            "KAFKA_PORT=9092",
            "DB_PORT=5432",
            "TITLE=\"App 1\"",
            "DESC=\"Desc 1\"",
        ]
    );
    Ok(())
}

#[test]
fn get_profiles() -> Result<(), Error> {
    let yaml = "
services:
  app: the-app
  psql:
    image: postgres
    profiles:
      - tools
  web:
    image: web-server
  app-provision:
    image: app
    profiles:
      - tools
      - provision
    ";
    let compose = ComposeYaml::new(&yaml)?;
    let profiles = compose.get_profiles_names();
    assert_eq!(profiles, Some(vec!["provision", "tools"]));
    Ok(())
}

#[test]
fn get_profiles_vector_notation() -> Result<(), Error> {
    let yaml = "
services:
  app: the-app
  app-provision:
    image: app
    profiles: [tools, provision]
    ";
    let compose = ComposeYaml::new(&yaml)?;
    let profiles = compose.get_profiles_names();
    assert_eq!(profiles, Some(vec!["provision", "tools"]));
    Ok(())
}

#[test]
fn get_profiles_none() -> Result<(), Error> {
    let yaml = "
volumes:
  data:
    driver: local
    ";
    let compose = ComposeYaml::new(&yaml)?;
    let profiles = compose.get_profiles_names();
    assert!(profiles.is_none());
    Ok(())
}

#[test]
fn get_images() -> Result<(), Error> {
    let yaml = "
services:
  app0: the-app
  app:
    image: app
  web:
    image: namespace.server.com/image:master
  psql:
    image: postgres:16.1
    profiles:
      - tools
  nginx:
    image: nginx:stable
  app-provision:
    image: app
    profiles:
      - tools
      - provision
  app-with-another-tag:
    image: app:1.0
    ";
    let compose = ComposeYaml::new(&yaml)?;
    let images = compose.get_images(None, None);
    assert_eq!(
        images,
        Some(vec![
            "app".to_string(),     // used twice, but included once
            "app:1.0".to_string(), // same image but with different version
            "namespace.server.com/image:master".to_string(),
            "nginx:stable".to_string(),
            "postgres:16.1".to_string(),
        ])
    );
    Ok(())
}

#[test]
fn get_images_filter_by_tag() -> Result<(), Error> {
    let yaml = "
services:
  app:
    image: app
  web:
    image: namespace.server.com/image:master
  psql:
    image: postgres:16.1
  nginx:
    image: nginx:master
  app-provision:
    image: app
  app-with-another-tag:
    image: app:1.0
    ";
    let compose = ComposeYaml::new(&yaml)?;
    let images = compose.get_images(Some("master"), None);
    assert_eq!(
        images,
        Some(vec![
            "namespace.server.com/image:master".to_string(),
            "nginx:master".to_string(),
        ])
    );
    Ok(())
}

#[test]
fn get_images_none() -> Result<(), Error> {
    let yaml = "
volumes:
  data:
    driver: local
    ";
    let compose = ComposeYaml::new(&yaml)?;
    let images = compose.get_images(None, None);
    assert!(images.is_none());
    Ok(())
}

#[test]
fn get_service_depends() -> Result<(), Error> {
    let yaml = r#"
services:
  app:
    image: the-app
    depends_on:
      - x
  postgres:
    image: postgres
  app1:
    image: some-image
    ports:
      - 8000:8000
    depends_on:
      - postgres
      - app
    "#;
    let compose = ComposeYaml::new(&yaml)?;
    let app1 = compose.get_service("app1").expect("app1 not found");
    let depends_on = compose.get_service_depends_on(&app1);
    assert_eq!(
        depends_on.unwrap_or(Vec::default()),
        vec!["postgres", "app"]
    );
    Ok(())
}

#[test]
fn get_service_depends_array_notation() -> Result<(), Error> {
    let yaml = r#"
services:
  app:
    image: the-app
    depends_on: [x, postgres]
  postgres:
    image: postgres
  app1:
    image: some-image
    ports:
      - 8000:8000
    depends_on:
      - postgres
      - app
    "#;
    let compose = ComposeYaml::new(&yaml)?;
    let app1 = compose.get_service("app").expect("app not found");
    let depends_on = compose.get_service_depends_on(&app1);
    assert_eq!(depends_on.unwrap_or(Vec::default()), vec!["x", "postgres"]);
    Ok(())
}

#[test]
fn get_service_depends_with_conditions() -> Result<(), Error> {
    let yaml = r#"
services:
  app:
    image: the-app
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
  postgres:
    image: postgres
    "#;
    let compose = ComposeYaml::new(&yaml)?;
    let app1 = compose.get_service("app").expect("app not found");
    let depends_on = compose.get_service_depends_on(&app1);
    assert_eq!(
        depends_on.unwrap_or(Vec::default()),
        vec!["postgres", "redis"]
    );
    Ok(())
}

#[test]
fn get_service_no_depends() -> Result<(), Error> {
    let yaml = r#"
services:
  app:
    image: the-app
  postgres:
    image: postgres
    "#;
    let compose = ComposeYaml::new(&yaml)?;
    let app1 = compose.get_service("app").expect("app not found");
    let depends_on = compose.get_service_depends_on(&app1);
    assert!(depends_on.is_none());
    Ok(())
}
