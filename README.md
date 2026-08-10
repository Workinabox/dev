# dev

The developer and orchestrator repo of workinabox.

## Commands

- `monitor` shows live organization status across all ten org repos (`.github`, `backend`, `frontend`, `app`, `website`, `sw-dev-team`, `iac`, `dev`, `docs`, `assets`)
- `release` tags and coordinates a synchronized release across the release subset (`RELEASE_REPOS` in `src/org.rs`): `.github`, `dev`, `backend`, `frontend`, `website`, `app`, `assets`. `sw-dev-team` versions independently (its own semver + Python wheel scheme) and `docs`/`iac` do not cut synchronized tags, so they are monitored but not release-tagged.

## Local stack

Run the whole app (Postgres + backend + frontend) locally with Docker Compose, from the
workspace root (the directory holding the sibling repos):

```sh
docker compose -f dev/local/docker-compose.yml up
```

See [local/README.md](local/README.md) for ports, the first-run notes, where data is stored,
and how to reset.

## GitHub Token

The `monitor` and `release` commands call the GitHub API at runtime. They read a Personal Access Token from your shell environment:

```sh
export GITHUB_WORKINABOX_TOKEN=ghp_yourtoken
```

Without it, the monitor will run but hit GitHub's unauthenticated rate limit (60 req/hour). The release command will fail if the token is missing and `--dry-run` is not set.

Required scopes: `repo` read for monitor, `repo` write for releases.
