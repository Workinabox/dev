# dev

The developer and orchestrator repo of workinabox.

## Commands

- `monitor` shows live organization status across `.github`, `dev`, `ui`, `backend`, and `app`
- `release` tags and coordinates synchronized releases across the sibling repos

## GitHub Token

The `monitor` and `release` commands call the GitHub API at runtime. They read a Personal Access Token from your shell environment:

```sh
export GITHUB_WORKINABOX_TOKEN=ghp_yourtoken
```

Without it, the monitor will run but hit GitHub's unauthenticated rate limit (60 req/hour). The release command will fail if the token is missing and `--dry-run` is not set.

Required scopes: `repo` read for monitor, `repo` write for releases.
