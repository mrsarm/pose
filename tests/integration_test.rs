/// The following tests are all marked as "ignore" to not delay tests execution,
/// but running the tests with the `--ignored` flag will make them to be executed,
/// (or use `make test-integration`).
use docker_pose::{ComposeYaml, DockerCommand, ReplaceTag, Verbosity};
use pretty_assertions::assert_eq;
use regex::Regex;
use serde_yaml::Error;
use serial_test::serial;

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
    let replace_tag = ReplaceTag {
        tag: "16.2".to_string(),
        tag_filter: None,
        ignore_unauthorized: true,
        no_slug: false,
        offline: false,
        verbosity: Verbosity::default(),
        progress_verbosity: Verbosity::Quiet,
        threads: 4,
    };
    let images = compose.get_images(None, Some(&replace_tag));
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
    let replace_tag = ReplaceTag {
        tag: "8".to_string(),
        tag_filter: Some((Regex::new(r"mysql").unwrap(), true)),
        ignore_unauthorized: true,
        no_slug: false,
        offline: false,
        verbosity: Verbosity::default(),
        progress_verbosity: Verbosity::Quiet,
        threads: 2,
    };
    let images = compose.get_images(None, Some(&replace_tag));
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

#[test]
#[ignore]
fn get_config_with_remote_tag_and_filter() -> Result<(), Error> {
    let yaml = r#"
services:
  postgres:
    image: postgres:7
  mysql:
    image: mysql:7
  rabbitmq:
    image: rabbitmq
    "#;
    let expected_yaml = r#"
services:
  postgres:
    image: postgres:7
  mysql:
    image: mysql:8
  rabbitmq:
    image: rabbitmq
    "#;
    let replace_tag = ReplaceTag {
        tag: "8 ".to_string(), // the white space will be trimmed when slug is used
        // Exclude postgres
        tag_filter: Some((Regex::new(r"postgres").unwrap(), false)),
        ignore_unauthorized: true,
        no_slug: false,
        offline: false,
        verbosity: Verbosity::default(),
        progress_verbosity: Verbosity::Quiet,
        threads: 2,
    };
    let mut compose = ComposeYaml::new(&yaml)?;
    compose.update_images_tag(&replace_tag);
    let new_yaml = compose.to_string();
    assert!(new_yaml.is_ok());
    assert_eq!(expected_yaml.to_string().trim(), new_yaml.unwrap().trim());
    Ok(())
}

#[test]
#[ignore]
#[serial]
fn get_config_with_local_tag() -> Result<(), Error> {
    // first pull in advance the image:tag desired
    let command = DockerCommand::new(Verbosity::default());
    command
        .pull_image("hello-world:linux", false, false)
        .unwrap();

    let yaml = r#"
services:
  hello-world:
    image: hello-world
    "#;
    let expected_yaml = r#"
services:
  hello-world:
    image: hello-world:linux
    "#;
    let replace_tag = ReplaceTag {
        tag: "linux".to_string(),
        tag_filter: None,
        ignore_unauthorized: true,
        no_slug: false,
        offline: false,
        verbosity: Verbosity::default(),
        progress_verbosity: Verbosity::Quiet,
        threads: 2,
    };
    let mut compose = ComposeYaml::new(&yaml)?;
    compose.update_images_tag(&replace_tag);
    let new_yaml = compose.to_string();
    assert!(new_yaml.is_ok());
    assert_eq!(expected_yaml.to_string().trim(), new_yaml.unwrap().trim());
    Ok(())
}

#[test]
#[ignore]
#[serial]
fn get_config_only_with_local_tag() -> Result<(), Error> {
    // first pull in advance the image:tag desired
    let command = DockerCommand::new(Verbosity::default());
    command
        .pull_image("hello-world:latest", false, false)
        .unwrap();

    let yaml = r#"
services:
  hello-world:
    image: hello-world:linux
  postgres:
    image: homeassistant/home-assistant:2024.3
    "#;

    // homeassistant/home-assistant:latest exists, but not locally
    let expected_yaml = r#"
services:
  hello-world:
    image: hello-world:latest
  postgres:
    image: homeassistant/home-assistant:2024.3
    "#;
    let replace_tag = ReplaceTag {
        tag: "latest".to_string(),
        tag_filter: None,
        ignore_unauthorized: true,
        no_slug: false,
        offline: true, // don't pull from remote docker registry
        verbosity: Verbosity::default(),
        progress_verbosity: Verbosity::Quiet,
        threads: 2,
    };
    let mut compose = ComposeYaml::new(&yaml)?;
    compose.update_images_tag(&replace_tag);
    let new_yaml = compose.to_string();
    assert!(new_yaml.is_ok());
    assert_eq!(expected_yaml.to_string().trim(), new_yaml.unwrap().trim());
    Ok(())
}
