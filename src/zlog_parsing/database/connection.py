"""MySQL connection construction and shared database types."""

import logging
import os
from typing import Any

import pymysql
import pymysql.cursors
from dotenv import load_dotenv

load_dotenv()

Connection = pymysql.Connection
Row = dict[str, Any]


def get_db_connection() -> Connection:
    """Create a configured autocommit MySQL connection."""
    ssl_options = None
    if os.getenv("DB_SSL_DISABLED", "false").lower() not in ("1", "true", "yes"):
        ssl_options = {"ca": "/etc/ssl/cert.pem"}
    try:
        return pymysql.connect(
            host=os.getenv("DB_HOST"),
            port=int(os.getenv("DB_PORT", "3306")),
            user=os.getenv("DB_USERNAME"),
            password=os.getenv("DB_PASSWORD"),
            database=os.getenv("DB_NAME"),
            autocommit=True,
            ssl=ssl_options,
            ssl_verify_identity=ssl_options is not None,
            cursorclass=pymysql.cursors.DictCursor,
        )
    except pymysql.MySQLError as exc:
        logging.error("Error connecting to the database: %s", exc)
        raise
