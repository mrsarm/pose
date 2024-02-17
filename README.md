Pose: a Docker Com-"pose" CLI
=============================

`pose` is a command line tool to play with :whale: Docker Compose files.

For now only supports listing some elements from a compose file:

```bash
$ pose list services
sales-service
postgres
$ pose list volumes
sales-data
pg-data
$ pose list envs postgres
PORT=5432
POSTGRES_PASSWORD=password
```

It looks for the compose file following the [spec](https://github.com/compose-spec/compose-spec/blob/master/spec.md#compose-file)
as `docker compose` does, or you can specify the filename as following: `pose -f another.yaml list services`.

Execute `pose --help` for more options, but don't expect too much, it's just a
project I made to have fun with Rust.

## Use Cases

Pose can be helpful when working with large compose files, with dozens of definitions,
where looking for something or summarize it can involve more work than without using pose.

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

This is useful when trying to access to services ran with Docker Compose, and
then you need to access them from the browser, or from an app outside Docker.

#### List environment variables from a service

It's specially useful when you want to centralize in one place all the environment
variables used across all services for local development, but at some point you need
to set up and run one or more services outside Docker. E.g.:

```bash
# Check the the services' environment variables
$ pose list envs sales-services
PORT=3000
DATABASE_URL=postgresql://sales:pass@postgres:5432/sales_dev
# Export them before run the service outside Docker
$ export $(pose list envs sales-services)
# Run the service, the envs have been set
$ yarn start
...
Webserver listening at http://localhost:3000
```

You can also export as an `.env` file the environment variables
of any service:

```bash
$ pose list envs portal-webapp > .env
```

## Install

Like any Rust project, install the binary `pose` in your system with:

```bash
$ cargo install docker-pose
```

(Yes, the package name in Crates.io is `docker-pose`, not `pose`).

Or from the source, after cloning the source code, go to the folder and
execute ` cargo install --path .` or `make install` (normally it will
install the binary in the `~/.cargo/bin` folder).

### Binary Download

Binaries are made available each release for Linux, Mac and Windows.

Download the binary on the [release](https://github.com/mrsarm/pose/releases) page.

Once downloaded, untar the file:

```bash
$ tar -xvf pose*.tar.gz
```

Check for the execution bit:

```bash
$ chmod +x pose
```

and then execute `pose`:

```bash
$ ./pose
```

Include the directory Pose is in, in your [PATH Variable](https://www.baeldung.com/linux/path-variable)
if you wish to be able to execute it anywhere, or move Pose to a directory already
included in your `$PATH` variable, like `$HOME/.local/bin`.

### Build and run tests

There is a `Makefile` that can be used to execute most of the development tasks,
like `make release` that executes `cargo build --release`, so check out the file
even if you don't use `make` to see useful commands.

#### Tests

- Rust tests: `make test`.
- Linter: `make lint`.
- Format checker: `make fmt-check`.
- Shell tests: `make test-cmd`. They are written in Shell script using 
  the test framework [Bats](https://bats-core.readthedocs.io).
- Rust integration tests: `make test-integration`. **Very slow**, they execute
  tests to check functionality that involves _pose → docker → docker registry_.
- Run all the tests at once: `make test-all-fast`, including all the above,
  except integrations one.
- Run **all the tests** at once: `make test-all`, include all tests, it's the
  equivalent of what CI executes on each push to GitHub.


If you get an error like `make: ./tests/bats/bin/bats: Command not found`,
it’s because you cloned the project without the `--recurse-submodules` git argument.
Execute first `git submodule update --init` to clone the submodules within the `pose` folder.


## About

**Source**: https://github.com/mrsarm/pose

**Authors**: (2022-2024) Mariano Ruiz <mrsarm (at) gmail.com>

**License**: GPL-3
