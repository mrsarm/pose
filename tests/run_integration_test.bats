setup() {
    load 'test_helper/bats-support/load'
    load 'test_helper/bats-assert/load'
}

teardown_file() {
    rm -f get-never.yaml
    rm -f ci-check.yaml
    rm -f compose-remote-check.yaml
}

@test "can list images with remote tag" {
    run target/debug/pose --verbose -f tests/compose-remote-check.yaml \
        list images --tag 3.1.1 --tag-filter "regex=mrsarm/"
    assert_success
    assert_output --partial "DEBUG: docker compose -f tests/compose-remote-check.yaml config"
    # django-mongotail:3.1.1 is checked, and it's found
    assert_output --partial "DEBUG: docker manifest inspect --insecure mrsarm/mongotail:3.1.1"
    assert_output --partial "DEBUG: manifest for image mrsarm/mongotail:3.1.1 ... found"
    # django-coleman:3.1.1 is checked, although it doesn't exist in the docker registry
    assert_output --partial "DEBUG: docker manifest inspect --insecure mrsarm/django-coleman:3.1.1"
    refute_output --partial "DEBUG: manifest for image mrsarm/django-coleman:3.1.1 ... found"
    # bitnami/kafka:3.1.1 exists, but is NOT checked because the filter
    refute_output --partial "DEBUG: docker manifest inspect --insecure bitnami/kafka:3.1.1"
    assert_output --partial "DEBUG: manifest for image bitnami/kafka ... skipped"

    # Output
    assert_output --partial "bitnami/kafka:3.0"
    assert_output --partial "mrsarm/django-coleman:1.0.1"
    # The only one changed:
    assert_output --partial "mrsarm/mongotail:3.1.1"
}

@test "can output config with remote tag" {
    run target/debug/pose --verbose -f tests/compose-remote-check.yaml \
        config --tag 3.1.1 --tag-filter "regex=mrsarm/"
    assert_success
    assert_output --partial "DEBUG: docker compose -f tests/compose-remote-check.yaml config"
    # django-mongotail:3.1.1 is checked, and it's found
    assert_output --partial "DEBUG: docker manifest inspect --insecure mrsarm/mongotail:3.1.1"
    assert_output --partial "DEBUG: manifest for image mrsarm/mongotail:3.1.1 ... found"
    # django-coleman:3.1.1 is checked, although it doesn't exist in the docker registry
    assert_output --partial "DEBUG: docker manifest inspect --insecure mrsarm/django-coleman:3.1.1"
    assert_output --partial "DEBUG: manifest for image mrsarm/django-coleman:3.1.1 ... not found"
    # bitnami/kafka:3.1.1 exists, but is NOT checked because the filter
    refute_output --partial "DEBUG: docker manifest inspect --insecure bitnami/kafka:3.1.1"
    assert_output --partial "DEBUG: manifest for image bitnami/kafka ... skipped"

    # Output
    assert_output --partial "image: bitnami/kafka:3.0"
    assert_output --partial "image: mrsarm/django-coleman:1.0.1"
    # The only one changed:
    assert_output --partial "image: mrsarm/mongotail:3.1.1"
}

@test "can get a file with another name" {
    run target/debug/pose get https://raw.githubusercontent.com/mrsarm/pose/main/tests/compose-remote-check.yaml -o ci-check.yaml
    assert_success
    assert_output --partial "DEBUG: Downloading https://raw.githubusercontent.com/mrsarm/pose/main/tests/compose-remote-check.yaml ... found"
    [ -f ci-check.yaml ]
}

@test "can get a file" {
    run target/debug/pose get https://raw.githubusercontent.com/mrsarm/pose/main/tests/compose-remote-check.yaml
    assert_success
    assert_output --partial "DEBUG: Downloading https://raw.githubusercontent.com/mrsarm/pose/main/tests/compose-remote-check.yaml ... found"
    [ -f compose-remote-check.yaml ]
}

@test "can get a file from URL generated from script" {
    run target/debug/pose get https://raw.githubusercontent.com/mrsarm/pose/never-exist/tests/compose-remote-check.yaml never-exist:main -o get-never.yaml
    assert_success
    assert_output --partial "DEBUG: Downloading https://raw.githubusercontent.com/mrsarm/pose/never-exist/tests/compose-remote-check.yaml ... not found"
    assert_output --partial "DEBUG: Downloading https://raw.githubusercontent.com/mrsarm/pose/main/tests/compose-remote-check.yaml ... found"
    [ -f get-never.yaml ]
}

@test "can get a file and not found it" {
    run target/debug/pose get https://raw.githubusercontent.com/mrsarm/pose/never-exist/tests/compose-remote-check.yaml
    assert_failure
    assert_output --partial "DEBUG: Downloading https://raw.githubusercontent.com/mrsarm/pose/never-exist/tests/compose-remote-check.yaml ... not found"
    assert_output --partial "ERROR: Download failed"
}

@test "can get a file and not found it and not get the scripted version as well" {
    run target/debug/pose get https://raw.githubusercontent.com/mrsarm/pose/never-exist/tests/compose-remote-check.yaml never-exist:not-exist-as-well
    assert_failure
    assert_output --partial "DEBUG: Downloading https://raw.githubusercontent.com/mrsarm/pose/never-exist/tests/compose-remote-check.yaml ... not found"
    assert_output --partial "DEBUG: Downloading https://raw.githubusercontent.com/mrsarm/pose/not-exist-as-well/tests/compose-remote-check.yaml ... not found"
    assert_output --partial "ERROR: Download failed"
}
