#!/home/michael/.pyenv/shims/python
# parse_logs.py

from psconnect import get_db_connection, insert_into, select_from, delete_from, Connection, Row, table_schemas
from zlog_queue import get_last_processed_id
import time

conn = get_db_connection()


def parse_log(log: Row) -> None:
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
        insert_into(conn, row, 'push')
        try:
            insert_into(conn, row, 'event_log')
        except Exception as e:
            if "Duplicate entry" in str(e):
                pass
        


def fetch_pm_table(conn: Connection) -> list[Row]:
    with conn.cursor() as cursor:
        cursor.execute("SELECT * FROM pm_table")
        return cursor.fetchall()

def pm_update(log: Row) -> None:
    pm_table = fetch_pm_table(conn)
    if log['window'] == log['nick'] and log['window'][0] != '#':
        if not any(row['window'] == log['window'] and row['nick'] == log['nick'] for row in pm_table):
            insert_into(conn, log, 'pm_table')


def main() -> None:
    conn = get_db_connection()
    try:
        last_processed_id = 28000000
        while True:
            logs = select_from(conn, "logs_queue", last_processed_id, desc=False)
            if not logs:
                time.sleep(10)  # Sleep for some time before checking for new logs
                continue
            for log in logs:
                parse_log(log)
                pm_update(log)
                delete_from(conn, 'logs_queue', {"id": log["id"]})
                print(f"Processed log {log['id']}")
                last_processed_id = log["id"]
    except Exception as e:
        print(f"An error occurred: {e}")


if __name__ == "__main__":
    main()
