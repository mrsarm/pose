/// The following tests are all marked as "ignore" to not delay tests execution,
/// but running the tests with the `--ignored` flag will make them to be executed,
/// (or use `make test-integration`).
use docker_pose::{ComposeYaml, RemoteTag, Verbosity};
use pretty_assertions::assert_eq;
use regex::Regex;
use serde_yaml::Error;

#[test]
#[ignore]
fn get_images_with_remote_tag() -> Result<(), Error> {
    let yaml = "
services:
  postgres:
    image: postgres:16.1
  psql:
    image: postgres:16.1
  nginx:
    image: nginx
  rabbitmq:
    image: rabbitmq:3
    ";
    let compose = ComposeYaml::new(&yaml)?;
    let images = compose.get_images(
        None,
        Some(RemoteTag {
            remote_tag: "16.2".to_string(),
            remote_tag_filter: None,
            ignore_unauthorized: true,
            verbosity: Verbosity::default(),
        }),
    );
    assert_eq!(
        images,
        Some(vec![
            "nginx".to_string(),
            "postgres:16.2".to_string(),
            "rabbitmq:3".to_string(),
        ])
    );
    Ok(())
}

#[test]
#[ignore]
fn get_images_with_remote_tag_and_filter() -> Result<(), Error> {
    let yaml = "
services:
  postgres:
    image: postgres:7
  mysql:
    image: mysql:7
    ";
    let compose = ComposeYaml::new(&yaml)?;
    let images = compose.get_images(
        None,
        Some(RemoteTag {
            remote_tag: "8".to_string(),
            remote_tag_filter: Some(Regex::new(r"mysql").unwrap()),
            ignore_unauthorized: true,
            verbosity: Verbosity::default(),
        }),
    );
    assert_eq!(
        images,
        Some(vec![
            "mysql:8".to_string(),
            // There is postgres:8, but was skipped with the regex filter
            "postgres:7".to_string(),
        ])
    );
    Ok(())
}