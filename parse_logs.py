#!/home/michael/.pyenv/shims/python
# parse_logs.py

from psconnect import get_db_connection, insert_into, select_from, delete_from, Connection, Row
from zlog_queue import get_last_processed_id
import time

import logging
from logging.handlers import RotatingFileHandler

def setup_logging() -> None:
    """
    Sets up logging for the script, including handlers for error and debug logs.
    """
    # Create a logger object
    logger = logging.getLogger()
    logger.setLevel(logging.DEBUG)  # Set to debug level to capture all messages

    # Create handlers for different log levels
    error_handler = RotatingFileHandler('error.log', maxBytes=10000, backupCount=5)
    error_handler.setLevel(logging.ERROR)
    error_handler.setFormatter(logging.Formatter('%(asctime)s - %(levelname)s - %(message)s'))

    debug_handler = RotatingFileHandler('debug.log', maxBytes=10000, backupCount=5)
    debug_handler.setLevel(logging.DEBUG)
    debug_handler.setFormatter(logging.Formatter('%(asctime)s - %(levelname)s - %(message)s'))

    # Add handlers to the logger
    logger.addHandler(error_handler)
    logger.addHandler(debug_handler)


conn = get_db_connection()


def parse_log(log: Row) -> None:
    """
    Parses a log entry and inserts it into the 'push' and 'event_log' tables if it meets certain criteria.
    """
    if log['type'] in ['msg', 'action']\
          and log['nick'] != 'BytesAndCoffee'\
          and 'bytesandcoffee' in log['message'].lower()\
          and not (log['window'] == '#reddit-sysadmin'\
                   and '<bytesandcoffee>' in log['message'].lower())\
          or log['window'].lower() == log['nick'].lower():
        # map log to push and event_log tables schemas
        row = {
            "id": log["id"],
            "user": log["user"],
            "network": log["network"],
            "window": log["window"],
            "type": log["type"],
            "nick": log["nick"],
            "message": log["message"]
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
                pass
        


def fetch_pm_table(conn: Connection) -> list[Row]:
    """
    Fetches all entries from the 'pm_table'.
    """
    with conn.cursor() as cursor:
        cursor.execute("SELECT * FROM pm_table")
        return cursor.fetchall()

def pm_update(log: Row) -> None:
    """
    Updates the 'pm_table' with a log entry if it meets certain criteria.
    """
    pm_table = fetch_pm_table(conn)
    if log['window'] == log['nick'] and log['window'][0] != '#':
        if not any(row['window'] == log['window'] and row['nick'] == log['nick'] for row in pm_table):
            try:
                insert_into(conn, log, 'pm_table')
            except Exception as e:
                logging.error("Failed to insert log %s into pm_table: %s", log["id"], e)


def main() -> None:
    """
    Main function that sets up logging, processes logs from the 'logs_queue' table, and updates the 'pm_table'.
    """
    setup_logging()
    try:
        conn = get_db_connection()
        last_processed_id = get_last_processed_id(conn) or 28000000
        while True:
            logs = select_from(conn, "logs_queue", last_processed_id, desc=False)
            if not logs:
                time.sleep(10)  # Sleep for some time before checking for new logs
                continue
            for log in logs:

                parse_log(log)
                pm_update(log)
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
