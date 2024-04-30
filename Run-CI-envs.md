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
to finally allow you to merge them all, or keep making changes without affect
the `latest` version (or whatever you call it) until all `new-tracking-field`
images from the different affected apps are ready to move into stage / prod.

Bellow you will find a Compose file example and examples of how to execute pose,
and at the end of the guide there is an example of how to write a GitHub Action
workflow with Docker Compose and pose to run E2E tests and release your images.

## Distributed Apps

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
and can also perform some actions, other integration tests can be shell scripts, written
in a scripting language like Python that make calls to your API, then check the result
and continues making more calls based on the results.

In the example above the integration tests could be written with Playwright (or Selenium)
to test the webapp in a headless browser (service `e2e`), so they were written
and stored in the image `mrsarm/e2e`.

**compose.yaml** file:

```yaml
services:
  web:
    image: mrsarm/web
    ports:
      - "8080:8080"
    depends_on: [api]
  # CI test task
  e2e:
    image: mrsarm/e2e
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
api api-worker postgres rabbitmq web e2e
$ pose list -p oneline images
mrsarm/api mrsarm/api-worker mrsarm/web mrsarm/e2e postgres:15 rabbitmq:3
```

There are 4 images starting with the prefix "mrsarm/" that are used for our services
_api_, _api-worker_ and _web_ (and the CI task _e2e_), while the other images
_postgres:15_ and _rabbitmq:3_ are DB services used by our services.

We can run all services and then the CI tests with `docker compose run e2e`,
so in what way is useful `pose` other than just allow us to list services or images?
well, now imagine that you are developing a new feature in the API to add a new field,
and that feature will be used by the webapp and the worker as well, but you need to
develop the feature across your apps one by one, each on its corresponding git repo in
a feature branch, then release the feature in the docker registry, e.g. you start
developing the feature in the `mrsarm/api` repo under the `client-vat-field` branch.
When you push something to the branch, your CI environment will create a new image
`mrsarm/api:client-vat-field`. Then you do the same for the webapp, the resulting
image is`mrsarm/web:client-vat-field`, and so on. So in the `compose.yaml` file you
can replace the following:

```yaml
services:
  web:
    image: mrsarm/api   # <-- change this with...
```

With:

```yaml
services:
    web:
    image: "mrsarm/api:${GITHUB_REF_NAME:-latest}"
```

GitHub Actions set the environment variable `GITHUB_REF_NAME` with the name of the branch
the CI task is running against to, so making the change above, `docker compose ...` will
replace at runtime the expression `mrsarm/web:${GITHUB_REF_NAME:-latest}` with
`mrsarm/web:client-vat-field`, producing the expected result of executing the tests on the
image desired, while when running the same compose file locally, the expression will be
turned into `mrsarm/web:latest` because the env variable `GITHUB_REF_NAME` doesn't exist.

All right, right?

#### The problem

The problem is, when you move forward with the rest of the apps where you add the feature,
how your CI environment distinguishes between the services that already
have the feature developed and published in the docker registry and
the ones don't? if you add the suffix `${GITHUB_REF_NAME:-latest}` to all the
images in the compose file (not to the DB images like postgres though), it will work as long
as all exist in the docker registry, but if let's say `mrsarm/api-worker:client-vat-field`
was not developed and published yet, `docker compose up` will exit with an
error like `manifest for mrsarm/api-worker:client-vat-field not found`.

#### Pose to the rescue

In the example above you only need to run the services with the tag `client-vat-field`
if the tag exists in the registry to be pulled by compose, otherwise keep using the
same `latest` tag set in the compose file, or whatever tag is set. You can achieve this with
the command `pose config` using the `--tag TAG` argument, that like
`docker compose config` it outputs a new compose file but pre-processing it according to
the arguments received. The following will output a new `ci.yaml` file with the desired
output, while showing in the terminal (or the CI logs) what is doing while fetching the
information from the docker registry:

```shell
pose config --tag "$GITHUB_REF_NAME" --progress -o ci.yaml
```

The remote progress will look like the following:

```
DEBUG: manifest for image mrsarm/web:client-vat-field ... found
DEBUG: manifest for image mrsarm/e2e:client-vat-field ... not found
DEBUG: manifest for image mrsarm/api-worker:client-vat-field ... not found
DEBUG: manifest for image postgres:client-vat-field ... not found
DEBUG: manifest for image mrsarm/api:client-vat-field ... found
DEBUG: manifest for image rabbitmq:client-vat-field ... not found
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
  e2e:
    image: mrsarm/e2e
    command: ["yarn", "test:ci"]
    depends_on: [web]
    profiles: ["e2e"]
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
docker compose -f ci.yaml run e2e
```

Note the `-f ci.yaml` argument used to ask compose to use the new compose file
instead of the original `compose.yaml` file.

### Advance options

There are more options that you can see with `pose config --help`, but there
are two special arguments that can help the process to speed up the execution.

#### Threads

The process of checking all the images your compose file has can
be slow, specially in a big app composed of dozens of apps.
The argument `--threads NUM` allows to specify the max number of parallel
threads to be used when fetching the images info from the remote registry. 
The default is 8 threads, and can be increased up to 32, but be carefully with
it, a high number can lead the docker registry to start responding with errors
related with rate limits reached.

#### Offline mode

With the flag `--offline` pose will check whether the tag passed exists or not
only in your local registry, useful for local testing.

Normally you will build images locally on each local repo and you may want to run
the tests in the same way CI does. If you also followed the convention of naming your
releases with the same name as the branch, you can use `pose slug` that outputs the
current name of the branch but post-processed to avoid issues with not allowed
chars like "/" that is turned into "-" (pose slugify any argument passed to
`-t`, `--tag` anyway, unless `--no-slug` is passed as well).

```shell
pose config -t "$(pose slug)" --tag-filter regex='mrsarm/' -o ci.yaml
```

#### Filters

The other argument that allows to speed up the process (and avoid rate limits
from the docker registry) is `--tag-filter FILTER`, filtering _in_ or _out_
what images to check whether a remote tag exists or not when replacing images.
`FILTER` can be an expression like `regex=NAME` (`=` → _filter ‒ in_) or `regex!=EXPR`
(`!=` → _filter ‒ out_), where `EXPR` is a regex expression. In our example, all the apps we
build start with the `mrsarm/` prefix, while other services like the DB ones don't,
so the best way to check only our apps while ignoring the rest when replacing the tag in
the image field of each service is using the argument `--tag-filter regex='mrsarm/'`.
The resulting `ci.yaml` will be identical than not using the filter at all, because it's
unlikely and even undesired to have an official Postgres image `postgres:client-vat-field`,
but more importantly, the execution in our CI pipeline will be much faster.

```
pose config -t "$(pose slug $GITHUB_REF_NAME)" --tag-filter regex='mrsarm/' -o ci.yaml --progress

DEBUG: manifest for image postgres ... skipped 
DEBUG: manifest for image rabbitmq ... skipped
DEBUG: manifest for image mrsarm/web:client-vat-field ... found
DEBUG: manifest for image mrsarm/api-worker:client-vat-field ... not found
DEBUG: manifest for image mrsarm/api:client-vat-field ... found
DEBUG: manifest for image mrsarm/e2e:client-vat-field ... not found
```

Use a _filter ‒ out_ expression when not all images follow certain convention like
in our example where all company's image start with the `mrsarm/` prefix, but at least
you know what are the images you don't want to be checked, so a regex expression using
`regex!=` could be as follows to achieve the same result: `postgres|rabbitmq`. Because
it's an _exclusion_ expression, all images with the string `postgres` or `rabbitmq`
on it will be ignored when replacing tags:

```shell
pose config -t "$GITHUB_REF_NAME" --tag-filter regex!='postgres|rabbitmq' -o ci.yaml --progress
```

#### Installing pose in a CI environment

Pose can be installed just downloading the right binary from GitHub, and unpacking it. Here
is the example for GitHub Actions:

```yaml
    - name: Install pose
      run: |
          wget https://github.com/mrsarm/pose/releases/download/0.4.0/pose-0.4.0-x86_64-unknown-linux-gnu.tar.gz -O - \
          | tar -xz
```

The command is available in the same folder your code is available, so you have to call it
with `./pose`.

#### Slugify branch name

Git branches can have names like "feature-brand-color" that are compatible with image
tag names in Docker, but can also have names like "feature/brand-color" which is not,
because the symbol "/" is not allowed in tag names. You can get a "slug" version when
tagging an image:

```shell
user@linuxl:webapp [feature/brand-color] $ pose slug
feature-brand-color
```

Normally you will use pose in a script, and in CI environments, where normally there is
a checkout of the _HEAD_ of the branch but without the `.git` folder and the git
command available, the name of the branch is available in an environment variable, e.g.
in GitHub the env `$GITHUB_REF_NAME` has the name of the branch, so to set the tag
for an image you are building in CI you can use:

```shell
docker build -t "myapp:$(./pose slug $GITHUB_REF_NAME)" .
```

If you are going to use the tag name in many places, better to set it in a new env
variable, in GitHub Actions you do so with:

```yaml
    - name: Define $TAG variable
      run: echo "TAG=$(./pose slug $GITHUB_REF_NAME)" >> "$GITHUB_ENV"
```

#### Download file from a branch URL for CI builds

You may prefer to have your Docker Compose definition, and maybe some other resources
like .env files in one repo, so each of your apps can be E2E tested but with the
same `compose.yaml` file across all your repos. Actually having a repo with all the
E2E tests and the `compose.yaml` file is the recommended approach, but in order
to run the E2E tests in all the repos you need not only the Docker images published
in a registry, but at least the `compose.yaml` to run them, so `pose get` helps with
that, using a similar approach to `pose config`: it tries to download a given
file from one URL (branch URL), if not found, tries with a "fallback" URL (usually
the "master" branch).

Here is an example for GitHub, you have all your e2e tests in the repo `mrsarm/e2e`,
the default branch is `master`, and at the root of the repo is the `compose.yaml`
file, so the file can be downloaded at
https://raw.githubusercontent.com/mrsarm/e2e/master/compose.yaml , but at some point
when working in a branch `ux-fix` in the repo `mrsarm/web`, you may want to introduce
changes in the e2e tests as well, including perhaps the compose file, so you
want to run the e2e tests with all the images with the tag `ux-fix` if available,
and you want to run them all using the `compose.yaml` file at the e2e repo from a branch
with the same name if available as well, otherwise use the "master" version.
So here is the script that allows to specify the URL where to get the file, otherwise
a "script" in the form of _to_replace:replacer_ to modify the URL with the default branch:

```yaml
- name: Get compose.yaml
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  run: |
    ./pose get -H "Authorization: token $GITHUB_TOKEN" \
       "https://raw.githubusercontent.com/mrsarm/e2e/$TAG/compose.yaml" "$TAG:main"
```

`$TAG` has the name of the branch CI is running against (`ux-fix` from the example), if
https://raw.githubusercontent.com/mrsarm/e2e/ux-fix/compose.yaml returns
HTTP 404 (Not Found), following the expression `"$TAG:main"` (`ux-fix:main"`) pose will try to get the
file from https://raw.githubusercontent.com/mrsarm/e2e/master/compose.yaml (the "master" version).

#### `--no-docker` argument

The command `pose config` call to `docker config --no-interpolate --no-normalize` first
to pre-process the compose file, which is specially useful if you want to merge multiple
compose files (you can pass more than one compose file with the argument `-f`, `--file`),
but in old versions of `docker compose` the `config` command outputs the new compose file
removing first all the objects, including services that should not be executed or used
when running `docker compose up`, so any service with a `profile:` set is going to be
removed in the output (this doesn't happen with newer versions of compose). So if you
don't need to pass more than one compose file to pose, or you have an old version of
Compose, use the flag `--no-docker` so Pose skip the prep-processing of your compose
file with `docker compose config`.

```shell
pose --no-docker config [...]
```

This is the case for GitHub Action at the day of writing this section, and can be
the case for CI environments that don't ship the `docker` or the `docker compose` command
in the pod running the jobs as well.

### GitHub Action example

Here is a full example of a GitHub Action workflow that uses pose to run
the tests from the `compose.yaml` example from above. The configuration
is as follows:

- The workflow works in the fictional repo `mrsarm/web`. The file is stored
  in the filepath `.github/workflows/e2e.yml`. A similar workflow could be
  configured on each of the apps, just changing the image is built, but running
  the same E2E tests.
- There is a service `e2e` where E2E tests run, the name of the image
  and the repo where the code is stored is `mrsarm/e2e`. The `compose.yaml`
  with all the settings is also stored at the root of `mrsarm/e2e`.


```yaml
name: Build & E2E

on: [push]

jobs:

  build-test-release:
    name: Build, Test and Release

    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v4

    - name: Login to Docker Hub
      uses: docker/login-action@v3
      with:
        username: ${{ vars.DOCKERHUB_USERNAME }}
        password: ${{ secrets.DOCKERHUB_TOKEN }}

    - name: Install pose
      run: |
          wget https://github.com/mrsarm/pose/releases/download/0.4.0/pose-0.4.0-x86_64-unknown-linux-gnu.tar.gz -O - \
          | tar -xz

    - name: Define $TAG variable
      run: echo "TAG=$(./pose slug $GITHUB_REF_NAME)" >> "$GITHUB_ENV"
    - name: Print tag and image names
      run: echo -e "- TAG    -->  $TAG\n- IMAGE  -->  mrsarm/web:$TAG"

    - name: Build the Docker image
      run: docker build -t mrsarm/web:${TAG} .

    #
    # At this point you may want to run unit tests from mrsarm/web
    # before releasing and running the e2e tests ...
    #
    # -name: Run unit tests
    #  run: docker run ... "mrsarm/web:$TAG" ...

    - name: Release Docker image
      # Release before running the e2e tests is up to you, here it's
      # released first in case you still want the image available in the
      # registry, even if the e2e tests fail.
      run: docker push "mrsarm/web:$TAG"

    - name: Get compose.yaml
      # Remember the compose file is stored at the mrsarm/e2e repo,
      # so it can be shared across all apps. CI has to fetch it first from
      # the right repo and branch, or use the default branch "master" if the feature
      # branch $TAG doesn't exist in mrsarm/e2e.
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      run: |
        ./pose get -H "Authorization: token $GITHUB_TOKEN" \
             "https://raw.githubusercontent.com/mrsarm/e2e/$TAG/compose.yaml" "$TAG:main"

    - name: Build compose file for CI
      run: |
        ./pose --no-docker config -t $TAG --tag-filter regex=mrsarm/ --progress -o ci.yaml

    - name: Pull images
      run: docker compose -f ci.yaml pull
           && docker compose -f ci.yaml pull e2e   # services with profiles are not pulled by default

    - name: Run e2e tests
      run: docker compose -f ci.yaml run e2e

    # Tagging and releasing "latest" only happens when CI is running against the master
    # branch, and all tests executed before succeeded
    - name: Tag "latest"
      if: ${{ github.ref == 'refs/heads/master' }}
      run: docker tag "mrsarm/web:$TAG" mrsarm/web:latest
    - name: Release "latest"
      if: ${{ github.ref == 'refs/heads/master' }}
      run: docker push mrsarm/web:latest
```
