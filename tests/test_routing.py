import unittest
from unittest.mock import patch

from zlog_parsing import routing


class RoutingTests(unittest.TestCase):
    def test_matching_log_is_written_idempotently_for_recipient(self):
        log = {
            "id": 10,
            "user": "znc-user",
            "network": "libera",
            "window": "#ops",
            "type": "msg",
            "nick": "alice",
            "message": "database alert",
        }
        rules = {"michael": [{"type": "substring", "match": "alert"}]}

        with patch.object(routing, "insert_ignore_into") as insert:
            matched = routing.route_log(object(), rules, log)

        self.assertTrue(matched)
        self.assertEqual(insert.call_count, 2)
        event_row, event_table = insert.call_args_list[0].args[1:]
        push_row, push_table = insert.call_args_list[1].args[1:]
        self.assertEqual(event_table, "event_log")
        self.assertEqual(push_table, "push")
        self.assertEqual(event_row["recipient"], "michael")
        self.assertEqual(push_row, event_row)

    def test_non_message_log_is_not_routed(self):
        log = {"type": "join"}
        with patch.object(routing, "insert_ignore_into") as insert:
            self.assertFalse(routing.route_log(object(), {}, log))
        insert.assert_not_called()


if __name__ == "__main__":
    unittest.main()
