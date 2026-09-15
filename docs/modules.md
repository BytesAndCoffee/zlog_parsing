# Package layout

Application code lives under `src/zlog_parsing`. The top-level worker scripts
remain only as compatibility entry points.

## `zlog_parsing.database`

- `connection.py` creates configured MySQL connections and defines shared row types.
- `schemas.py` owns runtime row validation.
- `queries.py` contains bounded SQL helpers and user repositories.
- `introspection.py` is the optional schema inspection utility.

## `zlog_parsing.rules` and `zlog_parsing.routing`

`rules` contains pure rule validation and matching behavior. `routing` loads
rules for recipients and inserts matching rows idempotently into `event_log`
and `push`.

## `zlog_parsing.recovery`

`coordinator.py` handles the shared outage marker, one-time notifications,
hourly probing, and live-head cutover. `store.py` persists recovery timestamps
and catch-up jobs in the Compose-managed SQLite volume, independently of MySQL.

## `zlog_parsing.workers`

- `producer.py` advances the live source checkpoint and classifies late replay.
- `live_parser.py` drains `logs_queue` and routes current notifications.
- `catchup.py` replays persisted ranges only when live notifications are clear.

These remain separate processes so a stopped worker is visible to Docker and
cannot be hidden behind another healthy foreground loop.

See [rules.md](rules.md) for the stored filtering-rule format.
