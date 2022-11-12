Pose: a Docker Com-"pose" CLI
=============================

`pose` is a command line tool to play with Docker Compose files.

For now only support listing root elements from a compose file:

```bash
$ pose list services
sales-service
postgres
$ pose list volumes
sales-data
pg-data
```

It looks for the compose file following the [spec](https://docs.docker.com/compose/compose-file/#compose-file)
about the name convention, or you can specify the name as following: `pose list -f another.yaml services`.

Execute `pose --help` for more options, but don't expect too much,
it's just a project I made to have fun with Rust.

## About

**Source**: https://github.com/mrsarm/pose

**Authors**: (2022) Mariano Ruiz <mrsarm (at) gmail.com>

**License**: GPL-3
