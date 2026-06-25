# Local full-stack dev environment

One command brings up the whole stack — Postgres, the Rust backend, and the React frontend
(built and served by nginx):

```sh
docker compose -f dev/local/docker-compose.yml up --build
```

Run it from the workspace root (the directory holding `backend/`, `frontend/`, and `dev/`).

| Service   | URL / address                            | Notes |
|-----------|------------------------------------------|-------|
| frontend  | http://localhost:3000                    | the app — open this |
| backend   | https://localhost:8080                   | HTTPS with a self-signed cert (use `curl -k`) |
| backend   | ssh://localhost:2222                     | git SSH transport |
| db        | postgres://wiab:wiab@localhost:5432/wiab | Postgres 16 |
| mailpit   | http://localhost:8025                    | dev inbox — catches outgoing email (e.g. password-reset links). SMTP is internal on `mailpit:1025` |
| oidc-mock | http://localhost:9090                    | mock OIDC provider for enterprise SSO (off unless enabled). Discovery: `/default/.well-known/openid-configuration` |

The frontend calls the backend through nginx's `/api` proxy, so it talks to the real backend
(not the in-app stub).

## Starting and stopping

Run all of these from the workspace root:

```sh
docker compose -f dev/local/docker-compose.yml up          # start (foreground; Ctrl-C stops)
docker compose -f dev/local/docker-compose.yml up -d       # start detached (background)
docker compose -f dev/local/docker-compose.yml up --build  # rebuild images first (after a Dockerfile/nginx change)
docker compose -f dev/local/docker-compose.yml ps          # show status
docker compose -f dev/local/docker-compose.yml logs -f     # follow logs (append a service name, e.g. backend)
docker compose -f dev/local/docker-compose.yml down        # stop and remove containers (data + compile caches kept)
```

## First run is slow

The backend image compiles Rust plus the cmake-built native dependencies
(llama.cpp / ggml / whisper / mediasoup) the first time, which can take tens of minutes.
The compiled output and the cargo registry are cached in named volumes (`wiab-target`,
`wiab-cargo`), so later runs are fast. The frontend won't come up until the backend's
`/health` check passes.

## Data lives in the workspace

Postgres data and the hosted git repos are bind-mounted under `dev/local/.data/`
(gitignored), so they survive restarts:

```
dev/local/.data/postgres   # Postgres data directory
dev/local/.data/git        # bare git repos (WIAB_GIT_ROOT)
```

Migrations run automatically on backend boot (refinery). To start completely fresh, stop the
stack and clear the data — but **recreate the empty subdirectories**. On Docker Desktop / WSL2,
deleting the bind-mount source dirs themselves makes the next `up` fail with
`no such file or directory` (Docker can't find them to mount). The dirs are root-owned, so the
simplest no-`sudo` way is a throwaway root container that wipes and recreates them in one shot:

```sh
docker compose -f dev/local/docker-compose.yml down
docker run --rm -v "$PWD/dev/local/.data:/data" alpine \
  sh -c 'rm -rf /data/postgres /data/git && mkdir -p /data/postgres /data/git'
docker compose -f dev/local/docker-compose.yml up -d
```

Always clear **both** `postgres` and `git` together. A repo is stored in two paired places — a
row in the Postgres `repo` table and an on-disk `R-<n>.git` bare repo — so clearing only one
leaves orphaned repos and id collisions on re-create. (Don't use `docker compose down -v` for
this: it removes the `wiab-target`/`wiab-cargo` compile caches, not the bind-mounted data.)

On the next `up` the backend re-seeds the default org/owner and logs a fresh bootstrap token
(see [Logging in](#logging-in)).

## Logging in

On the first boot against an empty database, the backend seeds a default org + Owner user
and prints a one-time bootstrap access token. Grab it from the logs:

```sh
docker compose -f dev/local/docker-compose.yml logs backend | grep -i "bootstrap access token"
```

## Known limitations

WebRTC media (mediasoup) uses a dynamic UDP port range and an announced address, which don't
traverse Docker's bridge network. The REST API and the app UI work; live audio/video would
need host networking or announced-address tuning.

## Alternative: backend on the host

To iterate on backend code without rebuilding the container, `backend/scripts/run-pg.sh`
starts the same Postgres in Docker and runs the backend with `cargo run` on the host.
