# Local full-stack dev environment

One command brings up the whole stack — Postgres, the Rust backend, and the React frontend
(built and served by nginx):

```sh
docker compose -f dev/local/docker-compose.yml up --build
```

Run it from the workspace root (the directory holding `backend/`, `frontend/`, and `dev/`).

| Service  | URL / address                          | Notes |
|----------|----------------------------------------|-------|
| frontend | http://localhost:3000                  | the app — open this |
| backend  | https://localhost:8080                 | HTTPS with a self-signed cert (use `curl -k`) |
| backend  | ssh://localhost:2222                   | git SSH transport |
| db       | postgres://wiab:wiab@localhost:5432/wiab | Postgres 16 |

The frontend calls the backend through nginx's `/api` proxy, so it talks to the real backend
(not the in-app stub).

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
stack and delete that directory:

```sh
docker compose -f dev/local/docker-compose.yml down
sudo rm -rf dev/local/.data   # sudo: Postgres owns its data files as its own uid
```

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
