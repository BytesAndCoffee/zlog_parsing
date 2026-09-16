import unittest
from unittest.mock import Mock, call, patch

from zlog_parsing.workers import live_parser


class LiveParserTests(unittest.TestCase):
    def test_empty_queue_backoff_resets_after_work_arrives(self):
        row = {
            "id": 101,
            "created_at": "now",
            "user": "u",
            "network": "n",
            "window": "#w",
            "type": "msg",
            "nick": "n",
            "message": "m",
        }
        logger = Mock()

        with (
            patch.object(live_parser, "fetch_pm_table", return_value=[]),
            patch.object(live_parser, "load_user_rules", return_value={}),
            patch.object(
                live_parser,
                "select_from",
                side_effect=[[], [], [row], [], RuntimeError("stop")],
            ),
            patch.object(live_parser, "route_log"),
            patch.object(live_parser, "maybe_track_pm"),
            patch.object(live_parser, "delete_from"),
            patch.object(live_parser, "LIVE_QUEUE_POLL_SECONDS", 1),
            patch.object(live_parser, "LIVE_QUEUE_MAX_POLL_SECONDS", 5),
            patch.object(live_parser.time, "sleep") as sleep,
        ):
            with self.assertRaisesRegex(RuntimeError, "stop"):
                live_parser.process_session(Mock(), logger)

        self.assertEqual(sleep.call_args_list, [call(1), call(2), call(1)])


if __name__ == "__main__":
    unittest.main()
