import os
import tempfile
import unittest
from datetime import datetime
from unittest.mock import patch

import state_store


class StateStoreTests(unittest.TestCase):
    def test_catchup_progress_and_cutoff_are_durable(self):
        with tempfile.TemporaryDirectory() as directory:
            path = os.path.join(directory, "recovery.sqlite3")
            cutoff = datetime(2026, 8, 30, 12, 0, 0)
            with patch.object(state_store, "STATE_DB_PATH", path):
                state_store.set_recovery_cutoff(cutoff)
                state_store.add_catchup_job(10, 20, cutoff)
                job = state_store.get_pending_job()
                self.assertEqual(job["next_id"], 10)
                self.assertEqual(job["created_before"], cutoff)

                state_store.mark_job_running(job["id"])
                state_store.update_job_next_id(job["id"], 15)
                self.assertEqual(state_store.get_pending_job()["next_id"], 15)

                state_store.complete_job(job["id"])
                self.assertIsNone(state_store.get_pending_job())
                self.assertEqual(state_store.get_recovery_cutoff(), cutoff)


if __name__ == "__main__":
    unittest.main()
