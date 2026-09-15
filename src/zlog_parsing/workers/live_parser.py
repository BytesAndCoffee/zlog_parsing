#!/usr/bin/env python3
"""Primary worker for current/live notification traffic."""

import logging
import time
from typing import Optional

import pymysql

from zlog_parsing.routing import load_user_rules, route_log
from zlog_parsing.database import (
    Connection,
    Row,
    delete_from,
    get_db_connection,
    select_from,
)
from zlog_parsing.recovery import mark_database_sleeping, wait_for_recovery_gate
from zlog_parsing.config import (
    LIVE_QUEUE_MAX_POLL_SECONDS,
    LIVE_QUEUE_POLL_SECONDS,
    QUEUE_BATCH_SIZE,
)
from zlog_parsing.logging import build_worker_logger


def setup_logging() -> logging.Logger:
    """Build the live parser's rotating-file logger."""
    return build_worker_logger("live-parser", "debug.log", error_file="error.log")


def fetch_pm_table(conn: Connection) -> list[Row]:
    with conn.cursor() as cursor:
        cursor.execute("SELECT * FROM pm_table")
        return cursor.fetchall()


def maybe_track_pm(conn: Connection, log: Row, pm_cache: set[tuple[str, str]]) -> None:
    if log["window"] == log["nick"] and not str(log["window"]).startswith("#"):
        key = (str(log["window"]), str(log["nick"]))
        if key not in pm_cache:
            with conn.cursor() as cursor:
                cursor.execute(
                    "INSERT INTO pm_table (`window`, nick) VALUES (%s, %s)",
                    key,
                )
            pm_cache.add(key)


def process_session(conn: Connection, logger: logging.Logger) -> None:
    """Process queued logs, backing off polling while the queue is empty."""
    pm_cache = {(str(row["window"]), str(row["nick"])) for row in fetch_pm_table(conn)}
    user_rules = load_user_rules(conn)
    processed_batches = 0
    poll_seconds = LIVE_QUEUE_POLL_SECONDS

    while True:
        logs = select_from(conn, "logs_queue", base=0, limit=QUEUE_BATCH_SIZE)
        if not logs:
            time.sleep(poll_seconds)
            poll_seconds = min(poll_seconds * 2, LIVE_QUEUE_MAX_POLL_SECONDS)
            continue
        poll_seconds = LIVE_QUEUE_POLL_SECONDS
        for log in logs:
            route_log(conn, user_rules, log)
            maybe_track_pm(conn, log, pm_cache)
            delete_from(conn, "logs_queue", {"id": log["id"]})
            logger.info("Processed live log %s", log["id"])
        processed_batches += 1
        if processed_batches % 100 == 0:
            user_rules = load_user_rules(conn)


def main() -> None:
    logger = setup_logging()
    conn: Optional[Connection] = None
    while True:
        try:
            wait_for_recovery_gate()
            conn = get_db_connection()
            process_session(conn, logger)
        except pymysql.MySQLError as exc:
            logger.error(
                "Live parser paused for database outage: %s", type(exc).__name__
            )
            mark_database_sleeping("live-parser")
            wait_for_recovery_gate()
        finally:
            if conn:
                conn.close()
                conn = None


if __name__ == "__main__":
    main()
