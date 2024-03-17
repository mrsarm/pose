setup() {
    load 'test_helper/bats-support/load'
    load 'test_helper/bats-assert/load'
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
    run target/debug/pose --verbose -f tests/compose-remote-check.yaml --no-normalize \
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
