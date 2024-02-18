setup() {
    load 'test_helper/bats-support/load'
    load 'test_helper/bats-assert/load'
}

@test "can run --version" {
    run target/debug/pose --version
    assert_success
    assert_output --partial 'docker-pose 0.3'
}

@test "can run --help" {
    run target/debug/pose --help
    assert_success
    assert_output --partial 'Command line tool to play with 🐳 Docker Compose files.'
    assert_output --partial 'list    List objects found in the compose file'
    # ...
    assert_output --partial '-f, --file <FILENAMES>'
}

@test "can list services" {
    run target/debug/pose -f tests/compose.yaml list services
    assert_success
    assert_output --partial "app1"
    assert_output --partial "app2"
    assert_output --partial "postgres"
    refute_output --partial "nginx"
}

@test "can list services without docker" {
    run target/debug/pose --no-docker -f tests/compose.yaml list services
    assert_success
    assert_output --partial "app1"
    assert_output --partial "app2"
    assert_output --partial "postgres"
}

@test "can list services in one line" {
    run target/debug/pose -f tests/compose.yaml list -p oneline services
    assert_success
    assert_output --partial "app1 app2 postgres"
}

@test "can list services from multiple sources" {
    run target/debug/pose -f tests/compose.yaml -f tests/another.yml list services
    assert_success
    assert_output --partial "app1"
    assert_output --partial "app2"
    assert_output --partial "nginx"
    assert_output --partial "postgres"
}

@test "cannot list services from multiple sources without docker" {
    run target/debug/pose --no-docker -f tests/compose.yaml -f tests/another.yml list services
    assert_failure 2
    refute_output --partial "app1"
    assert_output --partial "ERROR: multiple '--file' arguments cannot be used with '--no-docker'"
}

@test "can list images" {
    run target/debug/pose --verbose -f tests/compose.yaml list images
    assert_success
    assert_output --partial "DEBUG: docker compose -f tests/compose.yaml config"
    assert_output --partial "another-image:2.0"
    assert_output --partial "postgres:15"
    assert_output --partial "some-image"
}

@test "can list images with tag filter" {
    run target/debug/pose --verbose -f tests/compose.yaml list images --filter tag=2.0
    assert_success
    assert_output --partial "DEBUG: docker compose -f tests/compose.yaml config"
    assert_output --partial "another-image:2.0"
    refute_output --partial "postgres:15"
    refute_output --partial "some-image"
}

@test "can list images without docker" {
    run target/debug/pose --verbose --no-docker -f tests/compose.yaml list images
    assert_success
    refute_output --partial "DEBUG: docker compose -f tests/compose.yaml config"
    assert_output --partial "another-image:2.0"
    assert_output --partial "postgres:15"
    assert_output --partial "some-image"
}

@test "can list envs" {
    run target/debug/pose -f tests/compose.yaml list envs postgres
    assert_success
    assert_output --partial "PORT=5432"
    assert_output --partial "POSTGRES_PASSWORD=password"
}

@test "can list envs without docker" {
    run target/debug/pose --no-docker -f tests/compose.yaml list envs postgres
    assert_success
    assert_output --partial "PORT=5432"
    assert_output --partial "POSTGRES_PASSWORD=password"
}

@test "can detect service does not exist" {
    run target/debug/pose -f tests/compose.yaml list envs mememe
    assert_failure 16
    assert_output --partial "ERROR: No such service found: mememe"
}

@test "can show when a command does not exist" {
    run target/debug/pose some-command
    assert_failure 2
    assert_output --partial "error: unrecognized subcommand 'some-command'"
}

@test "can detect file does not exist" {
    run target/debug/pose -f does-not-exist.yaml list services
    assert_failure 14
    assert_output --partial "does-not-exist.yaml: no such file or directory"
}

@test "can detect file does not exist without docker" {
    run target/debug/pose --no-docker -f does-not-exist.yaml list services
    assert_failure 14
    assert_output --partial "does-not-exist.yaml: no such file or directory"
}

@test "can detect invalid file" {
    run target/debug/pose -f Makefile list services
    assert_failure 15
    assert_output --partial "ERROR: calling compose"
    assert_output --partial "yaml:"
}

@test "can detect invalid file without docker" {
    run target/debug/pose --no-docker -f Makefile list services
    assert_failure 15
    refute_output --partial "ERROR: calling compose"
    assert_output --partial "could not find expected ':'"
}

@test "can output config using docker" {
    run target/debug/pose -f tests/compose.yaml config
    assert_success
    assert_output --partial "app1"
    assert_output --partial "app2"
    assert_output --partial "postgres"
    refute_output --partial "nginx"
    # Secrets are filter out by docker compose config
    # because are not used in the example
    refute_output --partial "secrets"
}

@test "can output config without docker" {
    run target/debug/pose --no-docker -f tests/compose.yaml config
    assert_success
    assert_output --partial "app1"
    assert_output --partial "app2"
    assert_output --partial "postgres"
    refute_output --partial "nginx"
    # Secrets are NOT filter out by config
    # because are not used in the example
    assert_output --partial "secrets"
}

@test "can output config managing interpolation" {
    # With interpolation (default)
    run target/debug/pose -f tests/with-interpolation.yml config
    assert_success
    assert_output --partial 'app:latest'
    # Without interpolation
    run target/debug/pose --no-interpolate -f tests/with-interpolation.yml config
    assert_success
    assert_output --partial 'app:${TAG:-latest}'
}
