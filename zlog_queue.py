#!/usr/bin/env python3
"""Compatibility entry point for the packaged live producer worker."""

from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).parent / "src"))

from zlog_parsing.workers.producer import main  # noqa: E402


if __name__ == "__main__":
    main()
