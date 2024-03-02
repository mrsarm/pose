# Run feature branches in a CI environment

Dockerized apps are published in the Docker registry in a similar way they are stored in
a VCS repository (git), with big SHA numbers pointing to different version of the
app (commit revision in git, digest in docker), and branches and tags pointing to a
specific commit in git, the same with tags in docker pointing to SHA digests.

So here are some rules that are pretty normal among tech organizations:

- **Git repositories**:
    - `main` branch is the development branch in git, and contains what normally is
      on the stage environment running.
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

**TODO**.
