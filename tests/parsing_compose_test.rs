use pose::{get_services, get_services_names};

#[test]
fn get_services_list() -> Result<(), serde_yaml::Error> {
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
    let map = serde_yaml::from_str(&yaml)?;
    let services_map = get_services(&map).unwrap();
    let services_names = get_services_names(&map);
    assert_eq!(services_map.len(), services_names.len());
    assert_eq!(services_names, vec!["app", "app1", "app2"]);
    Ok(())
}

#[test]
fn get_services_empty_list() -> Result<(), serde_yaml::Error> {
    let yaml = "
services: []
volumes:
  - no-body-cares
    ";
    let map = serde_yaml::from_str(&yaml)?;
    assert!(get_services(&map).is_none());
    assert!(get_services_names(&map).is_empty());
    Ok(())
}

#[test]
fn get_services_no_list() -> Result<(), serde_yaml::Error> {
    let yaml = "
volumes:
  - no-body-cares
    ";
    let map = serde_yaml::from_str(&yaml)?;
    assert!(get_services(&map).is_none());
    assert!(get_services_names(&map).is_empty());
    Ok(())
}
