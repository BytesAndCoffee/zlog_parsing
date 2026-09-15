import os
import tempfile
import unittest
from unittest.mock import patch

import recovery


class RecoveryTests(unittest.TestCase):
    def test_sleep_notification_is_emitted_once(self):
        with tempfile.TemporaryDirectory() as directory:
            marker = os.path.join(directory, "sleeping")
            with patch.object(
                recovery, "DATABASE_SLEEP_STATE_FILE", marker
            ), patch.object(recovery, "send_telegram", return_value=True) as send:
                self.assertTrue(recovery.mark_database_sleeping("producer"))
                self.assertFalse(recovery.mark_database_sleeping("parser"))
                send.assert_called_once_with("Database slept")

    def test_recovery_range_includes_already_queued_rows(self):
        self.assertEqual(recovery.calculate_recovery_start(200, 190), 190)
        self.assertEqual(recovery.calculate_recovery_start(200, None), 201)


if __name__ == "__main__":
    unittest.main()
