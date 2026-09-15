#!/usr/bin/env python3
"""Compatibility entry point for the packaged container health check."""

from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).parent / "src"))

from zlog_parsing.healthcheck import main  # noqa: E402


if __name__ == "__main__":
    raise SystemExit(main())
