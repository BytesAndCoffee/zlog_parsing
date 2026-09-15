# zlog parsing

This repository contains tools for parsing IRC logs and storing them in a MySQL database. It also provides helper scripts for queueing logs and interacting with the database.

## Getting Started

Clone the repository and install the package. Python 3.10 or later is required.

```sh
pip install -e .
```

Copy `.env.example` to `.env` and fill in your database credentials.

## Usage

The project installs three independently supervised workers:

- `zlog-producer` – moves new logs into `logs_queue`
- `zlog-live-parser` – applies rules and creates notification rows
- `zlog-catchup` – replays outage ranges behind live traffic

Reusable code lives under `src/zlog_parsing`, grouped into database, recovery,
rules/routing, and worker modules. The historical top-level Python scripts are
retained as compatibility entry points.
- `main.sh` – runs the queue and parser scripts together

Run the parser and queue in the background when developing:

```sh
zlog-producer &
zlog-live-parser &
zlog-catchup &
```

## Docker

A simple container definition is provided in `main.dockerfile`. Build and run with:

```sh
docker build -f main.dockerfile -t zlog_parsing .
docker run -d zlog_parsing
```

Production deployments should use `docker compose up -d --build`. The Compose
definition supplies `.env`, persists the `unless-stopped` policy in Git, and
enables the worker health check.

## Database sleep recovery

The first worker that detects a database outage sends one out-of-band
`Database slept` Telegram alert. The live producer checks again hourly. Once
the database returns, live processing jumps to the current head immediately.
Rows dated before the recovery timestamp—including late ZNC disk replay—are
stored in the local durable recovery volume; current-dated rows stay on the
live path. A secondary worker plays matching historical notifications back
only while the live push queue is empty.
## Filtering Rules

Each user record contains a `hotwords` JSON column storing a **list** of rule objects.
Rules decide which log lines are queued for that user. Even a single rule must be
wrapped in a JSON array. See [docs/rules.md](docs/rules.md) for details.


## Documentation

Additional documentation and module descriptions can be found in the `docs` directory:

- [Setup Instructions](docs/setup.md)
- [Usage Instructions](docs/usage.md)
- [Modules Documentation](docs/modules.md)
- [Filtering Rules](docs/rules.md)

## License

This project is licensed under the MIT License.
