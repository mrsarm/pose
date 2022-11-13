Pose: a Docker Com-"pose" CLI
=============================

`pose` is a command line tool to play with :whale: Docker Compose files.

For now only supports listing root elements from a compose file:

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

## Use Cases

Pose can be helpful when working with large compose files, with dozens of definitions,
where looking for something or summarize it can involve more work than without using pose: 

#### Find that service you don't remember exactly the name

If you have dozens of services defined, some of them even with similar names, can be hard
to look at the file and find the one you don't remember the name, then `pose list services`
come to the rescue ! it prints all on the standard output, so you can use something
like `pose list services | grep sales`. Although you can `cat compose.yaml | grep sales`,
with large files that can output a lot of undesired lines, e.g. lines with environment
variables where the `sales` string is on it, and so on.

#### Get a full list of hosts names for `/etc/hosts`

This is my favorite:

```bash
$ pose list -p oneline services
sales-service postgres redis nginx ...
```

The `-p oneline` (or `--pretty online`) prints the list in one line, separating each
item with a white space, why is it useful? you can then paste the output attached to
a local IP in your `/etc/hosts`. Following the example:

```
127.0.0.1   sales-service postgres redis nginx ...
```

This is useful when trying to access to services ran with Docker Compose (or not) and
then you need to access them from the browser, from an app outside Docker...

## Install

Like any Rust project, install the binary `pose` in your system with:

```bash
$ cargo install docker-pose
```

(Yes, the package name in Crates.io is `docker-pose`, not `pose`).

Or from the source, after cloning the source code, go to the folder and
execute ` cargo install --path .`.

## About

**Source**: https://github.com/mrsarm/pose

**Authors**: (2022) Mariano Ruiz <mrsarm (at) gmail.com>

**License**: GPL-3
