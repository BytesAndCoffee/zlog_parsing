"""Container health check for the three long-running workers."""

from pathlib import Path

EXPECTED = {"zlog_queue.py", "parse_logs.py", "catchup_logs.py"}
running = set()
for command_file in Path("/proc").glob("[0-9]*/cmdline"):
    try:
        command = command_file.read_bytes().replace(b"\x00", b" ").decode()
    except (OSError, UnicodeDecodeError):
        continue
    for script in EXPECTED:
        if script in command:
            running.add(script)

raise SystemExit(0 if running == EXPECTED else 1)
