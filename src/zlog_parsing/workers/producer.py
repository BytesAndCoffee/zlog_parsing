#!/usr/bin/env python3
"""Live-head producer with database sleep recovery."""

import logging
import time
from typing import Optional

import pymysql

from zlog_parsing.database import Connection, get_db_connection, select_from
from zlog_parsing.recovery import (
    database_is_sleeping,
    mark_database_sleeping,
    recover_database,
)
from zlog_parsing.config import QUEUE_BATCH_SIZE
from zlog_parsing.logging import build_worker_logger
from zlog_parsing.recovery.store import add_catchup_job, get_recovery_cutoff


def setup_logging() -> logging.Logger:
    """Build the live producer's rotating-file logger."""
    return build_worker_logger("live-producer", "debug.log", error_file="error.log")


def get_last_processed_id(conn: Connection) -> Optional[int]:
    with conn.cursor() as cursor:
        cursor.execute("SELECT tid FROM logs_id_track WHERE id = 1")
        result = cursor.fetchone()
        return int(result["tid"]) if result else None


def copy_new_logs(conn: Connection) -> int:
    """Atomically enqueue one bounded live batch and advance its checkpoint."""
    last_copied_id = get_last_processed_id(conn) or 28000000
    new_logs = select_from(
        conn,
        "logs",
        base=last_copied_id,
        limit=QUEUE_BATCH_SIZE,
    )
    if not new_logs:
        return last_copied_id

    recovery_cutoff = get_recovery_cutoff()
    replay_logs = (
        [log for log in new_logs if log["created_at"] < recovery_cutoff]
        if recovery_cutoff
        else []
    )
    live_logs = [log for log in new_logs if log not in replay_logs]

    conn.begin()
    try:
        with conn.cursor() as cursor:
            for log in live_logs:
                cursor.execute(
                    """
                    INSERT IGNORE INTO logs_queue
                        (id, created_at, user, network, `window`, type, nick, message)
                    VALUES
                        (%(id)s, %(created_at)s, %(user)s, %(network)s,
                         %(window)s, %(type)s, %(nick)s, %(message)s)
                    """,
                    log,
                )
            if replay_logs:
                replay_start = int(replay_logs[0]["id"])
                replay_end = int(replay_logs[-1]["id"])
                add_catchup_job(replay_start, replay_end, recovery_cutoff)
            last_copied_id = int(new_logs[-1]["id"])
            cursor.execute(
                "REPLACE INTO logs_id_track (id, tid) VALUES (1, %s)",
                (last_copied_id,),
            )
        conn.commit()
    except Exception:
        conn.rollback()
        raise
    return last_copied_id


def main() -> None:
    logger = setup_logging()
    conn: Optional[Connection] = None
    while True:
        try:
            if database_is_sleeping():
                if conn:
                    conn.close()
                conn = recover_database(logger)
            elif conn is None:
                conn = (
                    recover_database(logger)
                    if database_is_sleeping()
                    else get_db_connection()
                )
            last_copied_id = copy_new_logs(conn)
            logger.info("Live checkpoint: %s", last_copied_id)
            time.sleep(1)
        except pymysql.MySQLError as exc:
            logger.error("Database unavailable: %s", type(exc).__name__)
            mark_database_sleeping("live-producer")
            if conn:
                conn.close()
            conn = recover_database(logger)


if __name__ == "__main__":
    main()
