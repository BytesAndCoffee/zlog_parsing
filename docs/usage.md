# Usage

This guide describes how to run the scripts and what they do.

## Running the Scripts

The project installs three worker commands:

- `zlog-producer` – copies new logs from `logs` into `logs_queue`
- `zlog-live-parser` – processes `logs_queue` and writes routed rows
- `zlog-catchup` – throttles missed outage matches behind current traffic

Ensure you have configured your `.env` file before running the scripts. The parser and queue can be launched together with:

```sh
zlog-producer &
zlog-live-parser &
zlog-catchup &
```

You can also run `main.sh`, which supervises all three commands.

## Environment Variables

The `.env` file should define the following variables:

```
DB_HOST=host
DB_USERNAME=username
DB_PASSWORD=password
DB_NAME=database
```

## Dependencies

Install the package and its dependencies using `pip install -e .`.

## Expected Behavior

- `zlog_parsing.database` owns connections, row schemas, and SQL repositories.
- The live parser applies hotword rules and writes matching entries to `push` and `event_log`.
- The producer monitors `logs` and enqueues new log lines for processing.
- After a database sleep, the producer probes hourly and resumes at the current
  live head. Late rows dated before recovery are recorded as durable catch-up
  jobs in the local recovery volume instead of blocking current notifications.
- The catch-up worker waits for the live `push` queue to drain before replaying
  another match. `CATCHUP_NOTIFICATION_INTERVAL_SECONDS` controls its pace.
- `zlog_parsing.rules` validates and evaluates each user's hotword rules.

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
