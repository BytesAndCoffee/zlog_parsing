"""Runtime configuration for live processing and outage recovery."""

import os


def env_int(name: str, default: int) -> int:
    value = int(os.getenv(name, str(default)))
    if value < 1:
        raise ValueError(f"{name} must be positive")
    return value


DB_RETRY_SECONDS = env_int("DB_RETRY_SECONDS", 3600)
QUEUE_BATCH_SIZE = env_int("QUEUE_BATCH_SIZE", 1000)
LIVE_QUEUE_POLL_SECONDS = env_int("LIVE_QUEUE_POLL_SECONDS", 1)
LIVE_QUEUE_MAX_POLL_SECONDS = env_int("LIVE_QUEUE_MAX_POLL_SECONDS", 5)
if LIVE_QUEUE_MAX_POLL_SECONDS < LIVE_QUEUE_POLL_SECONDS:
    raise ValueError(
        "LIVE_QUEUE_MAX_POLL_SECONDS must be greater than or equal to "
        "LIVE_QUEUE_POLL_SECONDS"
    )
CATCHUP_BATCH_SIZE = env_int("CATCHUP_BATCH_SIZE", 250)
CATCHUP_NOTIFICATION_INTERVAL_SECONDS = env_int(
    "CATCHUP_NOTIFICATION_INTERVAL_SECONDS", 15
)
CATCHUP_IDLE_SECONDS = env_int("CATCHUP_IDLE_SECONDS", 30)
PUSH_DRAIN_POLL_SECONDS = env_int("PUSH_DRAIN_POLL_SECONDS", 2)
DATABASE_SLEEP_STATE_FILE = os.getenv(
    "DATABASE_SLEEP_STATE_FILE", "/var/lib/zlog/database-sleeping"
)
STATE_DB_PATH = os.getenv("STATE_DB_PATH", "/var/lib/zlog/recovery.sqlite3")
