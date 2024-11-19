#!/home/michael/.pyenv/shims/python
# zlog_queue.py

import time
from typing import Optional
from psconnect import get_db_connection, insert_into, replace_into, select_from, Connection
import logging
from logging.handlers import RotatingFileHandler

def setup_logging() -> logging.Logger:
    """
    Sets up logging for the application, creating handlers for different log levels and adding them to the logger.
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
    return logger


# Starting ID is 28000000
def get_last_processed_id(conn: Connection) -> Optional[int]:
    """
    Fetches the last processed ID from the 'logs_id_track' table.
    """
    with conn.cursor() as cursor:
        cursor.execute("SELECT tid FROM logs_id_track")
        result = cursor.fetchone()
        return result['tid'] if result else None


def mark_as_processed(conn: Connection, last_id: Optional[int]) -> None:
    """
    Marks a log entry as processed by updating the 'logs_id_track' table.
    """
    if last_id is not None:
        replace_into(conn, {'id': 1, 'tid': last_id}, table="logs_id_track")


def copy_new_logs(conn: Connection, logger: logging.Logger) -> int:
    """
    Copies new log entries from the 'logs' table to the 'logs_queue' table and marks them as processed.
    """
    # Initialize or retrieve the last processed/copied ID
    last_copied_id = get_last_processed_id(conn) or 28000000

    try:
        # Select new log entries that haven't been processed/copied yet
        new_logs = select_from(conn, "logs", last_copied_id)

        # Copy the new logs into another table or process them as needed
        if new_logs:
            for log in new_logs:
                try:
                    insert_into(conn, log, table="logs_queue")
                    highest_id_in_batch = log['id']
                    mark_as_processed(conn, highest_id_in_batch)
                    last_copied_id = highest_id_in_batch
                except Exception as e:
                    logger.error(f"An error occurred while copying log {log['id']}: {e}")
                    raise e
    except Exception as e:
        logger.error(f"An error occurred while copying logs: {e}")
        raise e
    finally:
        return last_copied_id

# Example usage
def main() -> None:
    """
    Main function that sets up logging, copies new logs, and marks them as processed in a loop.
    """
    logger = setup_logging()
    try:
        conn = get_db_connection()
        while True:
            last_copied_id = copy_new_logs(conn, logger)
            logger.debug(f"Last copied ID: {last_copied_id}")
            time.sleep(10)  # Adjust the sleep time as necessary
    except Exception as e:
        logger.error("An error occurred: %s", e)
        raise e
        # Here, you could decide whether to retry the connection, or handle specific error types differently
    finally:
        if conn:
            conn.close()  # Ensure the connection is closed when the script is terminating

if __name__ == "__main__":
    main()
