# zlog parsing

This repository contains tools for parsing IRC logs and storing them in a MySQL database. It also provides helper scripts for queueing logs and interacting with the database.

## Getting Started

Clone the repository and install the dependencies listed in `requirements.txt`. Python 3.11 or later is required.

```sh
pip install -r requirements.txt
```

Copy `.env.example` to `.env` and fill in your database credentials.

## Usage

The project exposes several scripts:

- `psconnect.py` – helper functions for database access
- `parse_logs.py` – reads entries from `logs_queue` and stores them in the main tables
- `zlog_queue.py` – moves new logs into `logs_queue` so they can be processed
- `main.sh` – runs the queue and parser scripts together

Run the parser and queue in the background when developing:

```sh
python parse_logs.py &
python zlog_queue.py &
```

## Docker

A simple container definition is provided in `main.dockerfile`. Build and run with:

```sh
docker build -f main.dockerfile -t zlog_parsing .
docker run -d zlog_parsing
```
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
