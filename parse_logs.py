#!/home/michael/.pyenv/shims/python
# parse_logs.py

from psconnect import (
    get_db_connection,
    insert_into,
    select_from,
    delete_from,
    Connection,
    Row,
    fetch_users
)
from zlog_queue import get_last_processed_id
from rules import match_rule, fetch_rules

import time
import logging
from logging.handlers import RotatingFileHandler

# Global shared state
conn: Connection
users: list[str]
user_rules: dict[str, list[dict]]

def setup_logging() -> None:
    """
    Sets up logging for the script, including handlers for error and debug logs.
    """
    logger = logging.getLogger()
    logger.setLevel(logging.DEBUG)

    error_handler = RotatingFileHandler('error.log', maxBytes=10000, backupCount=5)
    error_handler.setLevel(logging.ERROR)
    error_handler.setFormatter(logging.Formatter('%(asctime)s - %(levelname)s - %(message)s'))

    debug_handler = RotatingFileHandler('debug.log', maxBytes=10000, backupCount=5)
    debug_handler.setLevel(logging.DEBUG)
    debug_handler.setFormatter(logging.Formatter('%(asctime)s - %(levelname)s - %(message)s'))

    logger.addHandler(error_handler)
    logger.addHandler(debug_handler)


def parse_log(log: Row) -> None:
    """
    Parses a log entry and inserts it into the 'push' and 'event_log' tables
    if it matches any rule for any user.
    """
    if log["type"] not in ["msg", "action"]:
        return

    for recipient, rules in user_rules.items():
        if any(match_rule(rule, log) for rule in rules):
            row = {
                "id": log["id"],
                "user": log["user"],
                "network": log["network"],
                "window": log["window"],
                "type": log["type"],
                "nick": log["nick"],
                "message": log["message"],
                "recipient": recipient
            }
            try:
                insert_into(conn, row, 'push')
            except Exception as e:
                logging.error("Failed to insert log %s into push: %s", log["id"], e)
            try:
                insert_into(conn, row, 'event_log')
            except Exception as e:
                logging.error("Failed to insert log %s into event_log: %s", log["id"], e)
                if "Duplicate entry" in str(e):
                    logging.debug("Duplicate entry: %s", log["id"])


def fetch_pm_table() -> list[Row]:
    """
    Fetches all entries from the 'pm_table'.
    """
    try:
        with conn.cursor() as cursor:
            cursor.execute("SELECT * FROM pm_table")
            return cursor.fetchall()
    except Exception as e:
        logging.error("Failed to fetch pm_table: %s", e)
        return []


def maybe_track_pm(log: Row, pm_cache: set[tuple[str, str]]) -> None:
    """
    If the log is a DM and hasn't been seen before, insert into pm_table and update cache.
    """
    if log["window"] == log["nick"] and not log["window"].startswith('#'):
        key = (log["window"], log["nick"])
        if key not in pm_cache:
            try:
                insert_into(conn, log, "pm_table")
                pm_cache.add(key)
            except Exception as e:
                logging.error("Failed to insert log %s into pm_table: %s", log["id"], e)



def main() -> None:
    """
    Main function that processes logs from 'logs_queue', handles rule matching and DM tracking.
    """
    setup_logging()
    try:
        global conn, users, user_rules
        conn = get_db_connection()
        pm_cache: set[tuple[str, str]] = set(
            (row["window"], row["nick"]) for row in fetch_pm_table()
        )
        last_processed_id = get_last_processed_id(conn) or 28000000
        users = fetch_users(conn)
        user_rules = {user: fetch_rules(conn, user) for user in users}

        while True:
            logs = select_from(conn, "logs_queue", last_processed_id, desc=False)
            if not logs:
                time.sleep(1)
                continue

            for log in logs:
                parse_log(log)
                maybe_track_pm(log, pm_cache)
                try:
                    delete_from(conn, 'logs_queue', {"id": log["id"]})
                except Exception as e:
                    logging.error("Failed to delete log %s from logs_queue: %s", log["id"], e)
                print(f"Processed log {log['id']}")
                last_processed_id = log["id"]

    except Exception as e:
        logging.error("An error occurred: %s", e)
        print(f"An error occurred: {e}")


if __name__ == "__main__":
    main()
