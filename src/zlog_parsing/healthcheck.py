"""Container health check for the three long-running workers."""

from pathlib import Path

EXPECTED = {
    "zlog_parsing.workers.producer",
    "zlog_parsing.workers.live_parser",
    "zlog_parsing.workers.catchup",
}


def find_running_workers(proc_root: Path = Path("/proc")) -> set[str]:
    """Return expected worker identifiers found in process command lines."""
    running = set()
    for command_file in proc_root.glob("[0-9]*/cmdline"):
        try:
            command = command_file.read_bytes().replace(b"\x00", b" ").decode()
        except (OSError, UnicodeDecodeError):
            continue
        for worker in EXPECTED:
            if worker in command:
                running.add(worker)
    return running


def main() -> int:
    """Return success only when all independently supervised workers exist."""
    return 0 if find_running_workers() == EXPECTED else 1


if __name__ == "__main__":
    raise SystemExit(main())
