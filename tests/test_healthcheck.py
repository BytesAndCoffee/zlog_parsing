import tempfile
import unittest
from pathlib import Path

from zlog_parsing.healthcheck import EXPECTED, find_running_workers


class HealthcheckTests(unittest.TestCase):
    def test_all_three_packaged_workers_are_detected(self):
        with tempfile.TemporaryDirectory() as directory:
            proc_root = Path(directory)
            for pid, worker in enumerate(sorted(EXPECTED), start=100):
                process = proc_root / str(pid)
                process.mkdir()
                (process / "cmdline").write_bytes(
                    f"python3\x00-m\x00{worker}\x00".encode()
                )

            self.assertEqual(find_running_workers(proc_root), EXPECTED)


if __name__ == "__main__":
    unittest.main()
