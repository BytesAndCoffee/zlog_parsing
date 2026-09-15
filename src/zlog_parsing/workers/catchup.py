"""Secondary worker for throttled playback of missed outage ranges."""

import logging
import time

import pymysql

from zlog_parsing.routing import load_user_rules, route_log
from zlog_parsing.database import Connection, get_db_connection, select_from
from zlog_parsing.recovery import mark_database_sleeping, wait_for_recovery_gate
from zlog_parsing.config import (
    CATCHUP_BATCH_SIZE,
    CATCHUP_IDLE_SECONDS,
    CATCHUP_NOTIFICATION_INTERVAL_SECONDS,
    PUSH_DRAIN_POLL_SECONDS,
)
from zlog_parsing.logging import build_worker_logger
from zlog_parsing.recovery.store import (
    complete_job,
    get_pending_job,
    mark_job_running,
    update_job_next_id,
)


def setup_logging() -> logging.Logger:
    """Build the catch-up worker's rotating-file logger."""
    return build_worker_logger("catchup", "catchup.log")


def wait_for_live_queue(conn: Connection) -> None:
    while True:
        with conn.cursor() as cursor:
            cursor.execute("SELECT COUNT(*) AS pending FROM push")
            if int(cursor.fetchone()["pending"]) == 0:
                return
        time.sleep(PUSH_DRAIN_POLL_SECONDS)


def process_job(conn: Connection, job: dict, logger: logging.Logger) -> None:
    mark_job_running(job["id"])
    rules = load_user_rules(conn)
    next_id = int(job["next_id"])
    end_id = int(job["end_id"])
    created_before = job["created_before"]

    while next_id <= end_id:
        rows = select_from(
            conn,
            "logs",
            base=next_id - 1,
            end=end_id,
            limit=CATCHUP_BATCH_SIZE,
        )
        if not rows:
            next_id = end_id + 1
            break
        for row in rows:
            if row["created_at"] >= created_before:
                next_id = int(row["id"]) + 1
                update_job_next_id(job["id"], next_id)
                continue
            wait_for_live_queue(conn)
            matched = route_log(conn, rules, row)
            next_id = int(row["id"]) + 1
            update_job_next_id(job["id"], next_id)
            if matched:
                time.sleep(CATCHUP_NOTIFICATION_INTERVAL_SECONDS)

    complete_job(job["id"])
    logger.info("Completed catch-up job %s", job["id"])


def main() -> None:
    logger = setup_logging()
    while True:
        conn = None
        try:
            wait_for_recovery_gate()
            conn = get_db_connection()
            job = get_pending_job()
            if job:
                process_job(conn, job, logger)
            else:
                time.sleep(CATCHUP_IDLE_SECONDS)
        except pymysql.MySQLError as exc:
            logger.error("Catch-up paused for database outage: %s", type(exc).__name__)
            mark_database_sleeping("catchup")
            wait_for_recovery_gate()
        finally:
            if conn:
                conn.close()


if __name__ == "__main__":
    main()
