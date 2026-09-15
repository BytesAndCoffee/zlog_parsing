#!/usr/bin/env python3
"""Compatibility entry point for the packaged schema introspection utility."""

from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).parent / "src"))

from zlog_parsing.database.introspection import (  # noqa: E402,F401
    convert_type,
    fetch_schema,
    main,
    print_schema,
)


if __name__ == "__main__":
    main()
