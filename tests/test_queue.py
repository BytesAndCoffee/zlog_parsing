import unittest
from datetime import datetime, timedelta
from unittest.mock import patch

import pymysql

import zlog_queue


class Cursor:
    def __init__(self, checkpoint=100):
        self.checkpoint = checkpoint
        self.executed = []

    def __enter__(self):
        return self

    def __exit__(self, *_args):
        return False

    def execute(self, sql, params=None):
        self.executed.append((sql, params))

    def fetchone(self):
        return {"tid": self.checkpoint}


class Connection:
    def __init__(self):
        self.cursor_value = Cursor()
        self.committed = False
        self.rolled_back = False

    def cursor(self):
        return self.cursor_value

    def begin(self):
        pass

    def commit(self):
        self.committed = True

    def rollback(self):
        self.rolled_back = True


class QueueTests(unittest.TestCase):
    def test_bounded_batch_is_enqueued_and_checkpointed_atomically(self):
        conn = Connection()
        rows = [
            {
                "id": 101,
                "created_at": "now",
                "user": "u",
                "network": "n",
                "window": "#w",
                "type": "msg",
                "nick": "n",
                "message": "m",
            }
        ]
        with patch.object(
            zlog_queue, "select_from", return_value=rows
        ) as select, patch.object(zlog_queue, "get_recovery_cutoff", return_value=None):
            checkpoint = zlog_queue.copy_new_logs(conn)
        self.assertEqual(checkpoint, 101)
        self.assertTrue(conn.committed)
        select.assert_called_once_with(
            conn, "logs", base=100, limit=zlog_queue.QUEUE_BATCH_SIZE
        )

    def test_database_errors_are_not_swallowed(self):
        conn = Connection()
        failure = pymysql.OperationalError(1105, "database slept")
        with patch.object(zlog_queue, "select_from", side_effect=failure):
            with self.assertRaises(pymysql.OperationalError):
                zlog_queue.copy_new_logs(conn)

    def test_late_replay_rows_become_catchup_instead_of_live(self):
        conn = Connection()
        cutoff = datetime(2026, 8, 30, 12, 0, 0)
        rows = [
            {
                "id": 101,
                "created_at": cutoff - timedelta(hours=1),
                "user": "u",
                "network": "n",
                "window": "#w",
                "type": "msg",
                "nick": "n",
                "message": "buffered",
            },
            {
                "id": 102,
                "created_at": cutoff + timedelta(seconds=1),
                "user": "u",
                "network": "n",
                "window": "#w",
                "type": "msg",
                "nick": "n",
                "message": "live",
            },
        ]
        with patch.object(zlog_queue, "select_from", return_value=rows), patch.object(
            zlog_queue, "get_recovery_cutoff", return_value=cutoff
        ), patch.object(zlog_queue, "add_catchup_job") as add_job:
            zlog_queue.copy_new_logs(conn)
        statements = [sql for sql, _params in conn.cursor_value.executed]
        self.assertEqual(
            sum("INSERT IGNORE INTO logs_queue" in sql for sql in statements), 1
        )
        add_job.assert_called_once_with(101, 101, cutoff)


if __name__ == "__main__":
    unittest.main()
