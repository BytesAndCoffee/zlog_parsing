"""Database sleep detection and recovery cutover coordination."""

import logging
import os
import time
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Callable, Optional

from notifier import send_telegram
from psconnect import Connection, get_db_connection
from settings import (
    DATABASE_SLEEP_STATE_FILE,
    DB_RETRY_SECONDS,
)
from state_store import add_catchup_job, set_recovery_cutoff


@dataclass(frozen=True)
class RecoveryRange:
    start_id: int
    end_id: int
    row_count: int
    created_before: datetime


def calculate_recovery_start(checkpoint: int, queue_first: Optional[int]) -> int:
    """Include any rows already queued before the outage in catch-up."""
    start_id = checkpoint + 1
    return min(start_id, queue_first) if queue_first is not None else start_id


def mark_database_sleeping(source: str) -> bool:
    """Persist the outage marker and alert exactly once across all workers."""
    Path(DATABASE_SLEEP_STATE_FILE).parent.mkdir(parents=True, exist_ok=True)
    try:
        descriptor = os.open(
            DATABASE_SLEEP_STATE_FILE,
            os.O_CREAT | os.O_EXCL | os.O_WRONLY,
            0o600,
        )
    except FileExistsError:
        return False
    with os.fdopen(descriptor, "w") as marker:
        marker.write(f"{int(time.time())} {source}\n")
    send_telegram("Database slept")
    return True


def database_is_sleeping() -> bool:
    return os.path.exists(DATABASE_SLEEP_STATE_FILE)


def wait_for_recovery_gate(sleep: Callable[[float], None] = time.sleep) -> None:
    """Keep non-coordinator workers paused until recovery cutover completes."""
    while database_is_sleeping():
        sleep(min(DB_RETRY_SECONDS, 60))


def get_log_head(conn: Connection) -> int:
    with conn.cursor() as cursor:
        cursor.execute("SELECT COALESCE(MAX(id), 0) AS head FROM logs")
        return int(cursor.fetchone()["head"])


def prepare_live_cutover(
    conn: Connection,
    head: int,
    created_before: Optional[datetime] = None,
) -> Optional[RecoveryRange]:
    """Advance live processing and persist the skipped range for catch-up."""
    with conn.cursor() as cursor:
        cursor.execute("SELECT tid FROM logs_id_track WHERE id = 1")
        checkpoint_row = cursor.fetchone()
        checkpoint = int(checkpoint_row["tid"]) if checkpoint_row else 28000000
        cursor.execute("SELECT MIN(id) AS first_id FROM logs_queue")
        queue_row = cursor.fetchone()
        queue_first = queue_row["first_id"] if queue_row else None
        if created_before is None:
            cursor.execute("SELECT UTC_TIMESTAMP() AS recovered_at")
            created_before = cursor.fetchone()["recovered_at"]

    start_id = calculate_recovery_start(
        checkpoint,
        int(queue_first) if queue_first is not None else None,
    )

    recovery_range = None
    set_recovery_cutoff(created_before)
    if start_id <= head:
        add_catchup_job(start_id, head, created_before)
        recovery_range = RecoveryRange(
            start_id=start_id,
            end_id=head,
            row_count=head - start_id + 1,
            created_before=created_before,
        )
    conn.begin()
    try:
        with conn.cursor() as cursor:
            cursor.execute("DELETE FROM logs_queue WHERE id <= %s", (head,))
            cursor.execute(
                "REPLACE INTO logs_id_track (id, tid) VALUES (1, %s)",
                (head,),
            )
        conn.commit()
    except Exception:
        conn.rollback()
        raise
    return recovery_range


def recover_database(
    logger: logging.Logger,
    sleep: Callable[[float], None] = time.sleep,
) -> Connection:
    """Probe hourly, establish a stable live head, and release paused workers."""
    while True:
        sleep(DB_RETRY_SECONDS)
        try:
            conn = get_db_connection()
            conn.ping(reconnect=False)
            head = get_log_head(conn)
            recovery_range = prepare_live_cutover(conn, head)
            try:
                os.unlink(DATABASE_SLEEP_STATE_FILE)
            except FileNotFoundError:
                pass
            if recovery_range:
                send_telegram(
                    "Database awake. Live notifications resumed; "
                    f"throttled catch-up queued for {recovery_range.row_count} rows."
                )
            else:
                send_telegram("Database awake. Live notifications resumed.")
            return conn
        except Exception as exc:
            logger.error(
                "Database still unavailable; checking again in %s seconds: %s",
                DB_RETRY_SECONDS,
                type(exc).__name__,
            )
            if "conn" in locals():
                try:
                    conn.close()
                except Exception:
                    pass
