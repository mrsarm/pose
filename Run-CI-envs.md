# Run feature branches in a CI environment

In this guide we'll see how `pose` can help you to test distributed apps
in CI environments, using Docker Compose to run the apps and integration
tests, while `pose` "enables" you to test different "feature" branches without
the need to release them, specially when the feature needs to be developed
across many of your apps (but not necessarily all of them).

Pose allows in CI environments to build on the fly a new compose file from
another, replacing tag versions from the images with a new remote
version (if exists), making it possible to develop a feature across
dockerized apps, tagged with a common name, e.g. `new-tracking-field`,
then test them all together in a CI environment with docker compose,
to finally merge them all or keep making changes without affect the `latest`
version (or whatever you call it) until all `new-tracking-field` images from the
different affected apps are ready to move into stage / prod.

## Introduction

Dockerized apps are published in the Docker registry in a similar way they are stored in
a VCS repository (git), with big SHA numbers pointing to different version of the
app (commit revision in git, digest in docker), and branches and tags pointing to a
specific commit in git, the same with tags in docker pointing to SHA digests.


### Git and Docker flows

Here are some rules that are pretty normal among tech organizations to manage
git repos and docker images across their apps:

- **Git repositories**:
    - `main` branch (or `master`) is the development branch in git, and contains what
      normally is on the stage environment running.
    - `prod` branch is the production branch in git, what normally is live in production.
    - `**?` other branches are considered "feature" branches, or branches to make fixes,
      they are created from the `main` branch, and eventually merged into `main`.
- **Docker registry**:
    - `latest` tag is the development tag, so whatever is pushed to `master` in git, is
      built in the CI environment into a docker image, and stored in the registry
      and tagged as `latest`. Then in a continues delivery workflow the image may
      be used to deploy the app version in stage automatically, or manually.
    - `prod` tag is production, so whatever is push to `prod` in git, is
      built in the CI environment and tagged as `prod`. Normally the deployment to
      the production environment is done manually.
    - `**?` other tags are "feature" tags made from its corresponding branches, they
      are not deployed automatically in any environment, but can be manually deployed
      into stage for testing before the feature branch is merged into `main`, or
      can be used with docker / docker compose to run them in a developer's computer
      or the CI environment for integration tests.
      **Here is when pose comes to help 💪** as we'll see below.

### Guided example without pose and the limitations

Now let's see an example where you have an application that is actually a distributed
app composed of many services (microservices, web server, workers, DBs...).
In the following example you have a compose file that allows to run all the services
at once, and a task that runs an integration test.

Normally an integration test can be a script that interacts with your app through a
browser in an automated way, testing some conditions are met when the webapp is loaded,
and can also perform some actions, other integration test can be shell scripts, written
in a scripting language like Python that make calls to your API, then check the result
and continues making more calls based on the results.

In the example above the integration tests could be written with Playwright (or Selenium)
to test the webapp in a headless browser (service `web-ci-tests`), so
they were written and stored in the same image used to store the webapp (image
`mrsarm/web`), but executed with a different command (`yarn test:ci`).

**compose.yaml** file:

```yaml
services:
  web:
    image: mrsarm/web
    ports:
      - "8080:8080"
    depends_on: [api]
  # CI test task
  web-ci-tests:
    image: mrsarm/web
    command: ["yarn", "test:ci"]
    depends_on: [web]
    profiles: [ci]   # because it has a profile, it doesn't run by default
  api:
    image: mrsarm/api
    ports:
      - "3000:3000"
    depends_on: [postgres, rabbitmq]
  api-worker:
    image: mrsarm/api-worker
    ports:
      - "3000:3000"
    depends_on: [postgres, rabbitmq]
  postgres:
    image: postgres:15
    ports:
      - "5432:5432"
    environment:
      - POSTGRES_PASSWORD=password
  rabbitmq:
    image: rabbitmq:3
    ports:
      - "5672:5672"
```

This is the list of services and images get by `pose`:

```shell
$ pose list -p oneline services
api api-worker postgres rabbitmq web web-ci-tests
$ pose list -p oneline images
mrsarm/api mrsarm/api-worker mrsarm/web postgres:15 rabbitmq:3
```

There are 3 images, all starting with the prefix "mrsarm/" that are used
for our services _api_, _api-worker_ and _web_ (and the CI task
_web-ci-tests_), while the other images _postgres:15_ and _rabbitmq:3_
are DB services used by our services.

We can run all services with: `docker compose up -d` in detached
mode, and then the CI tests with `docker compose run web-ci-tests`,
so in what way is useful `pose` other than just allow us to
list services or images? well, now imagine that you are developing
a new feature in the API to add a new field, and that feature
will be used by the webapp and the worker as well, but you
need to develop the feature across your apps one by one, each
on its corresponding git repo in a feature branch, then release  
the feature in the docker registry, e.g. you start developing
the feature in the `mrsarm/api` repo under the `client-vat-field`
branch. When you push something to the branch, your CI environment
will create a new image `mrsarm/api:client-vat-field`. Then
you do the same for the webapp, the resulting image is
`mrsarm/web:client-vat-field`, and so on. So in the `compose.yaml`
file you can replace the following:

```yaml
services:
  web:
    image: mrsarm/api   # <-- change this with...
```

With:

```yaml
services:
    web:
    image: "mrsarm/api:${GITHUB_REF:-latest}"
```

GitHub Actions set the environment variable `GITHUB_REF` with the name of the branch
the CI task is running against to, so making the change above, `docker compose ...` will
replace at runtime the expression `mrsarm/web:${GITHUB_REF:-latest}` with
`mrsarm/web:client-vat-field`, producing the expected result of executing the tests on the
image desired, while when running the same compose file locally, the expression will be
turned into `mrsarm/web:latest` because the env variable `GITHUB_REF` doesn't exist.

All right, right?

#### The problem

The problem is, when you move forward with the rest of the apps where you add the feature,
how your CI environment distinguishes between the services that already
have the feature developed and published in the docker registry and
the ones don't? if you add the suffix `${GITHUB_REF:-latest}` to all the
images (not to the DB images like postgres though), it will work as long as
all exists in the docker registry, but if let's say `mrsarm/api-worker:client-vat-field`
was not developed and published yet, `docker compose up` will exit with an
error like `manifest for mrsarm/api-worker:client-vat-field not found`.

#### Pose to the rescue

In the example above you only need to run the services with the tag `client-vat-field`
if the tag exists in the registry to be pulled of by compose, otherwise keep using the
same `latest` tag set in the compose file, or whatever tag is set. You can achieve this with
the command `pose config` using the `--remote-tag TAG` argument, that like
`docker compose config` it outputs a new compose file but pre-processing it according to
the arguments received. The following will output a new `ci.yaml` file with the desired
output, while showing in the terminal (or the CI logs) what is doing while fetching the
information from the docker registry:

```shell
pose config --remote-tag "$GITHUB_REF" --remote-progress -o ci.yaml
```

The remote progress will look like the following:

```
DEBUG: remote manifest for image mrsarm/web:client-vat-field ... found 
DEBUG: remote manifest for image mrsarm/api-worker:client-vat-field ... not found 
DEBUG: remote manifest for image postgres:client-vat-field ... not found 
DEBUG: remote manifest for image mrsarm/api:client-vat-field ... found 
DEBUG: remote manifest for image rabbitmq:client-vat-field ... not found 
```

In the progress shown we can see the images found with the tag `client-vat-field`
are `mrsarm/api` and `mrsarm/web`, while the others like `mrsarm/api-worker` don't,
so the resulting services definition in the new `ci.yaml` will be like:

```yaml
services:
  web:
    image: mrsarm/web:client-vat-field
    ports:
      - "8080:8080"
    depends_on: [api]
  web-ci-tests:
    image: mrsarm/web:client-vat-field
    command: ["yarn", "test:ci"]
    depends_on: [web]
  api:
    image: mrsarm/api:client-vat-field
    ports:
      - "3000:3000"
    depends_on: [postgres, rabbitmq]
  api-worker:
      image: mrsarm/api-worker
      ports:
          - "3000:3000"
      depends_on: [postgres, rabbitmq]
  postgres:
      image: postgres:15
      ports:
          - "5432:5432"
      environment:
          - POSTGRES_PASSWORD=password
  rabbitmq:
      image: rabbitmq:3
      ports:
          - "5672:5672"
```

Then the services and the tests can be executed with:

```shell
docker compose -f ci.yaml up -d
docker compose -f ci.yaml run web-ci-tests
```

Note the `-f ci.yaml` argument used to ask compose to use the new compose file
instead of the original `compose.yaml` file.

### Advance options

There are more options that you can see with `pose config --help`, but there
are two special arguments that can help the process to speed up the execution.

#### Threads

The process of checking all the images your compose file has can
be slow, specially in big app composed of dozens of apps.
The argument `--threads NUM` allows to specify the max number of parallel
threads to be used when fetching the images info from the remote registry. 
The default is 4 threads, and can be increased up to 32, but be carefully with
it, a high number can lead the docker registry to start responding with errors
related with rate limits reached.

#### Filters

The other argument that allows to speed up the process (and avoid rate limits
from the docker registry) is `--remote-tag-filter FILTER`, filtering _in_ or _out_
what images to check whether a remote tag exists or not when replacing images.
`FILTER` can be an expression like `regex=NAME` (`=` → _filter ‒ in_) or `regex!=EXPR`
(`!=` → _filter ‒ out_), where `EXPR` is a regex expression. In our example, all the apps we
build start with the `mrsarm/` prefix, while other services like the DBs one don't,
so the best way to check only our apps while ignoring the rest when replacing the tag in
the image field of each service is using the argument `--remote-tag-filter regex='mrsarm/'`.
The resulting `ci.yaml` will be identical than not using the filter at all, because it's
unlikely and even undesired to have an official Postgres image `postgres:client-vat-field`,
but more importantly, the execution in our CI pipeline will be much faster.

```
pose config --remote-tag "$GITHUB_REF" --remote-tag-filter regex='mrsarm/' -o ci.yaml --remote-progress

DEBUG: remote manifest for image postgres ... skipped 
DEBUG: remote manifest for image rabbitmq ... skipped
DEBUG: remote manifest for image mrsarm/web:client-vat-field ... found 
DEBUG: remote manifest for image mrsarm/api-worker:client-vat-field ... not found 
DEBUG: remote manifest for image mrsarm/api:client-vat-field ... found 
```

Use a _filter ‒ out_ expression when not all images follow certain convention like
in our example where all company's image start with the `mrsarm/` prefix, but at least
you know what are the images you don't want to be checked, so a regex expression using
`regex!=` could be as follows to achieve the same result: `postgres|rabbitmq`. Because
it's an _exclusion_ expression, all images with the string `postgres` or `rabbitmq`
on it will be ignored when replacing tags:

```shell
pose config --remote-tag "$GITHUB_REF" --remote-tag-filter regex!='postgres|rabbitmq' -o ci.yaml --remote-progress
```
