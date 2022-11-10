use pose::get_compose_map;

#[test]
fn parse_compose() -> Result<(), serde_yaml::Error> {
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
    let map = get_compose_map(&yaml)?;
    let services = &map["services"].as_mapping().unwrap();
    assert_eq!(services.keys().collect::<Vec<_>>().len(), 3);
    Ok(())
}
