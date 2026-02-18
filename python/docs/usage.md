# Usage

This guide describes how to run the scripts and what they do.

## Running the Scripts

The project consists of three main Python files:

- `psconnect.py` – utilities for connecting to the MySQL database
- `parse_logs.py` – processes entries from `logs_queue` and writes them to other tables
- `zlog_queue.py` – copies new logs from `logs` into `logs_queue`

Ensure you have configured your `.env` file before running the scripts. The parser and queue can be launched together with:

```sh
python parse_logs.py &
python zlog_queue.py &
```

You can also run `main.sh` which simply executes both commands.

## Environment Variables

The `.env` file should define the following variables:

```
DB_HOST=host
DB_USERNAME=username
DB_PASSWORD=password
DB_NAME=database
```

## Dependencies

Install dependencies listed in `requirements.txt` using `pip install -r requirements.txt`.

## Expected Behavior

- `psconnect.py` provides database helper functions.
- `parse_logs.py` reads logs from `logs_queue`, applies hotword rules and writes matching entries to `push` and `event_log`.
- `zlog_queue.py` monitors the `logs` table and enqueues new log lines for processing.
- `rules.py` contains helper functions for validating and evaluating the hotword rules stored for each user.

## Filtering Rules

Users define filtering rules as JSON objects stored in the `users.hotwords` column.
The value is a JSON **array** of rule objects even when only a single rule is defined.
Each rule determines whether a log line should be delivered to that user. See
[docs/rules.md](rules.md) for the full schema. A simple example:

```json
[
  {
    "type": "substring",
    "match": "error",
    "only_if": {"window": "#general"}
  }
]
```

The parser evaluates these rules for every log line and queues matches into the `push` table.
