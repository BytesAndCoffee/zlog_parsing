"""Runtime row schemas used to guard generic database writes."""

import logging
from datetime import datetime
from typing import Any

from zlog_parsing.database.connection import Row

ColumnSpec = tuple[type[Any], bool]

TABLE_SCHEMAS: dict[str, dict[str, ColumnSpec]] = {
    "logs": {
        "created_at": (datetime, False),
        "id": (int, False),
        "message": (str, True),
        "network": (str, True),
        "nick": (str, True),
        "type": (str, False),
        "user": (str, True),
        "window": (str, False),
    },
    "logs_id_track": {"id": (int, False)},
    "logs_queue": {
        "id": (int, False),
        "created_at": (datetime, False),
        "user": (str, True),
        "network": (str, True),
        "window": (str, False),
        "type": (str, False),
        "nick": (str, True),
        "message": (str, True),
    },
    "event_log": {
        "id": (int, False),
        "message": (str, True),
        "network": (str, False),
        "nick": (str, True),
        "type": (str, False),
        "user": (str, True),
        "window": (str, False),
    },
    "push": {
        "id": (int, False),
        "message": (str, True),
        "network": (str, False),
        "nick": (str, True),
        "type": (str, False),
        "user": (str, True),
        "window": (str, False),
    },
    "users": {
        "nickname": (str, False),
        "telegram_chat_id": (int, True),
        "hotwords": (list, True),
    },
}


def validate_schema(row: Row, table: str) -> bool:
    """Return whether a row contains valid values for a known table."""
    schema = TABLE_SCHEMAS.get(table)
    if schema is None:
        raise ValueError(f"Unknown table: {table}")
    for column, (column_type, nullable) in schema.items():
        if column not in row:
            if not nullable:
                logging.error("Column %s is missing from the row", column)
                return False
            continue
        if row[column] is None:
            if not nullable:
                logging.error("Column %s cannot be null", column)
                return False
        elif not isinstance(row[column], column_type):
            logging.error("Column %s must be of type %s", column, column_type.__name__)
            return False
    return True
