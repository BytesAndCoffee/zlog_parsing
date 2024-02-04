#!/home/michael/.pyenv/shims/python
# zlog_queue.py

import time
from typing import Optional
from psconnect import get_db_connection, insert_into, replace_into, select_from, Connection


# Starting ID is 28000000
def get_last_processed_id(conn: Connection) -> Optional[int]:
    with conn.cursor() as cursor:
        cursor.execute("SELECT tid FROM logs_id_track")
        result = cursor.fetchone()
        return result['tid'] if result else None


def mark_as_processed(conn: Connection, last_id: Optional[int]) -> None:
    if last_id is not None:
        replace_into(conn, {'id': 1, 'tid': last_id}, table="logs_id_track")


def copy_new_logs(conn: Connection) -> int:
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
                    print(f"An error occurred while copying log {log['id']}: {e}")
                    raise e
    except Exception as e:
        print(f"An error occurred while copying logs: {e}")
        raise e
    finally:
        return last_copied_id

# Example usage
def main() -> None:
    conn = None
    try:
        conn = get_db_connection()
        while True:
            last_copied_id = copy_new_logs(conn)
            print(f"Last copied ID: {last_copied_id}")
            time.sleep(10)  # Adjust the sleep time as necessary
    except Exception as e:
        print(f"An error occurred: {e}")
        raise e
        # Here, you could decide whether to retry the connection, or handle specific error types differently
    finally:
        if conn:
            conn.close()  # Ensure the connection is closed when the script is terminating

if __name__ == "__main__":
    main()
