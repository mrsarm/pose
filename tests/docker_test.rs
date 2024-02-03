use docker_pose::Verbosity::Verbose;
use docker_pose::{DockerCommand, Verbosity};
use serial_test::serial;
use std::env;

#[test]
fn run_docker_version() {
    let command = DockerCommand::new(Verbosity::default());
    let output = command.call_cmd(vec!["version"], false, true).unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.to_lowercase().contains("version"));
}

#[test]
fn run_docker_compose_version() {
    for verbosity in [Verbosity::Quiet, Verbose] {
        let command = DockerCommand::new(verbosity);
        let output = command
            .call_cmd(vec!["compose", "version"], false, true)
            .unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.to_lowercase().contains("version"));
    }
}

#[test]
#[serial]
fn run_missed_docker() {
    env::set_var("DOCKER_BIN", "docker1234");
    let command = DockerCommand::new(Verbosity::default());
    let result = command.call_cmd(vec!["version"], false, false);
    assert!(result.is_err()); // the message vary according to the OS
    env::set_var("DOCKER_BIN", "docker");
}

#[test]
fn run_docker_missed_file() {
    let command = DockerCommand::new(Verbosity::default());
    let output = command
        .call_cmd(
            vec!["compose", "-f", "doesnotexist.yaml", "up"],
            false,
            true,
        )
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.to_lowercase().contains("no such file or directory"));
}

#[test]
fn run_docker_config() {
    let command = DockerCommand::new(Verbose);
    let output = command.call_compose_config(None, false, true).unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // POSTGRES_PASSWORD=password was turned into...
    assert!(stdout.contains("POSTGRES_PASSWORD: password"));
}

#[test]
fn run_docker_config_multiple_files() {
    let command = DockerCommand::new(Verbose);
    let output = command
        .call_compose_config(
            Some(vec!["tests/compose.yaml", "tests/another.yml"]),
            false,
            true,
        )
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // VAR=val was turned into VAR: val
    assert!(stdout.contains("POSTGRES_PASSWORD: password"));
    assert!(stdout.contains("image: nginx"));
}

#[test]
fn run_docker_config_file_not_found() {
    let command = DockerCommand::new(Verbose);
    let output = command
        .call_compose_config(Some(vec!["does-not-exist.yml"]), false, true)
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("no such file or directory"));
}