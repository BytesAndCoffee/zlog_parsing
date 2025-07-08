#!/home/michael/.pyenv/shims/python
# psconnect.py

import os, pymysql, pymysql.cursors
from typing import Optional, List, Dict, Any
from datetime import datetime
from dotenv import load_dotenv
import logging
load_dotenv()

Connection = pymysql.Connection
table_schemas = {
    "logs": {
        "meta-schema": {
            "column": ["type", "nullable"]
        },
        "columns": [
            {"created_at": [datetime, False]},
            {"id": [int, False]},
            {"message": [str, True]},
            {"network": [str, True]},
            {"nick": [str, True]},
            {"type": [str, False]},
            {"user": [str, True]},
            {"window": [str, False]}
        ]
    },
    "logs_id_track": {
        "meta-schema": {
            "column": ["type", "nullable"]
        },
        "columns": [
            {"id": [int, False]}
        ]
    },
    "logs_queue": {
        "meta-schema": {
            "column": ["type", "nullable"]
        },
        "columns": [
            {"id": [int, False]},
            {"created_at": [datetime, False]},
            {"user": [str, True]},
            {"network": [str, True]},
            {"window": [str, False]},
            {"type": [str, False]},
            {"nick": [str, True]},
            {"message": [str, True]}
        ]
    },
    "event_log": {
        "meta-schema": {
            "column": ["type", "nullable"]
        },
        "columns": [
            {"id": [int, False]},
            {"message": [str, True]},
            {"network": [str, False]},
            {"nick": [str, True]},
            {"type": [str, False]},
            {"user": [str, True]},
            {"window": [str, False]}
        ]
    },
    "push": {
        "meta-schema": {
            "column": ["type", "nullable"]
        },
        "columns": [
            {"id": [int, False]},
            {"message": [str, True]},
            {"network": [str, False]},
            {"nick": [str, True]},
            {"type": [str, False]},
            {"user": [str, True]},
            {"window": [str, False]}
        ]
    },
    "event_log": {
        "meta-schema": {
            "column": ["type", "nullable"]
        },
        "columns": [
            {"id": [int, False]},
            {"message": [str, True]},
            {"network": [str, False]},
            {"nick": [str, True]},
            {"type": [str, False]},
            {"user": [str, True]},
            {"window": [str, False]}
        ]
    }
}

Row = Dict[str, Any]
logging.basicConfig(level=logging.ERROR, filename='error.log', filemode='a', 
                    format='%(asctime)s - %(levelname)s - %(message)s')

def get_db_connection() -> Connection:
    """
    Establishes a connection to the database using environment variables for configuration.
    Returns a pymysql.Connection object.
    """
    try:
        # Connect to the database using environment variables
        conn = pymysql.connect(
            host=os.getenv("DB_HOST"),
            user=os.getenv("DB_USERNAME"),
            password=os.getenv("DB_PASSWORD"),
            db=os.getenv("DB_NAME"),
            autocommit=True,
            ssl={"ca": "/etc/ssl/cert.pem"},
            ssl_verify_identity=True,
            cursorclass=pymysql.cursors.DictCursor
        )
        return conn
    except pymysql.MySQLError as e:
        # Log any connection errors
        logging.error(f"Error connecting to the database: {e}")
        raise e
        

def validate_schema(row: Row, table: str) -> bool:
    """
    Validates that a given row matches the schema for a specified table.
    Returns True if the row is valid, False otherwise.
    """
    schema = table_schemas[table]
    for column_spec in schema["columns"]:
        for col, specs in column_spec.items():
            col_type, nullable = specs
            if col not in row:
                if not nullable:
                    # Log an error if a non-nullable column is missing
                    logging.error(f"Column {col} is missing from the row")
                    return False
                else:
                    continue  # It's okay for nullable columns to be missing
            if row[col] is None and not nullable:
                # Log an error if a non-nullable column is null
                logging.error(f"Column {col} cannot be null")
                return False
            if row[col] is not None and not isinstance(row[col], col_type):
                # Log an error if the column type does not match
                logging.error(f"Column {col} must be of type {col_type.__name__}")
                return False
    return True

def insert_into(conn: pymysql.Connection, row: Row, table: str) -> None:
    """
    Inserts a row into a specified table in the database.
    Validates the row against the table schema before insertion.
    """
    if not validate_schema(row, table):
        raise ValueError("Invalid schema")
    cols = ', '.join(f'`{col}`' for col in row.keys())
    vals = ', '.join(f'%({col})s' for col in row.keys())
    sql = f'INSERT INTO `{table}` ({cols}) VALUES ({vals})'
    try:
        # Execute the insert statement
        with conn.cursor() as cursor:
            cursor.execute(sql, row)
            conn.commit()
    except pymysql.MySQLError as e:
        # Rollback the transaction in case of an error
        conn.rollback()
        logging.error(f"Error inserting into {table}: {e}")
        raise e


def replace_into(conn: pymysql.Connection, row: Row, table: str) -> None:
    """
    Replaces a row in a specified table in the database.
    Validates the row against the table schema before replacement.
    """
    if not validate_schema(row, table):
        raise ValueError("Invalid schema")
    cols = ', '.join(f'`{col}`' for col in row.keys())
    vals = ', '.join(f'%({col})s' for col in row.keys())
    sql = f'REPLACE INTO `{table}` ({cols}) VALUES ({vals})'
    try:
        # Execute the replace statement
        with conn.cursor() as cursor:
            cursor.execute(sql, row)
            conn.commit()
    except pymysql.MySQLError as e:
        # Rollback the transaction in case of an error
        conn.rollback()
        logging.error(f"Error replacing into {table}: {e}")


def select_from(
    conn: pymysql.Connection,
    table: str,
    base: int = 28000000,
    desc: bool = False,
    limit: Optional[int] = None,
) -> Optional[List[Dict[str, Any]]]:
    """Select rows from ``table`` where ``id`` is greater than ``base``.

    Parameters
    ----------
    conn : Connection
        Database connection.
    table : str
        Table name to query.
    base : int, optional
        Minimum ``id`` to select from, by default ``28000000``.
    desc : bool, optional
        Order results descending when ``True``.
    limit : Optional[int], optional
        Maximum number of rows to return. ``None`` for no limit.

    Returns
    -------
    Optional[List[Dict[str, Any]]]
        List of rows returned from the query or ``None`` on error.
    """
    try:
        # Execute the select statement
        with conn.cursor() as cursor:
            limit_clause = f" LIMIT {limit}" if limit else ""
            sql = (
                f"SELECT * FROM {table} WHERE id > {base} "
                f"ORDER BY id {'DESC' if desc else 'ASC'}{limit_clause}"
            )
            cursor.execute(sql)
            return cursor.fetchall()
    except pymysql.MySQLError as e:
        logging.error(f"Error selecting from {table}: {e}")
        return None


def delete_from(conn: pymysql.Connection, table: str, conditions: dict) -> None:
    """
    Deletes rows from a specified table in the database based on given conditions.
    """
    if not conditions:
        logging.error("Conditions required for deletion to prevent accidental table wipe.")
        raise ValueError("Conditions required for deletion to prevent accidental table wipe.")
    
    where_clause_parts = []
    params = []
    for column, value in conditions.items():
        # Build the WHERE clause for the delete statement
        where_clause_parts.append(f"`{column}` = %s")
        params.append(value)
    
    where_clause = " AND ".join(where_clause_parts)
    
    sql = f"DELETE FROM `{table}` WHERE {where_clause}"
    
    try:
        # Execute the delete statement
        with conn.cursor() as cursor:
            cursor.execute(sql, params)
            conn.commit()
    except pymysql.MySQLError as e:
        # Rollback the transaction in case of an error
        conn.rollback()
        logging.error(f"Error deleting from {table}: {e}")
        raise e


def delete_many(conn: pymysql.Connection, table: str, ids: List[int]) -> None:
    """Delete multiple rows from ``table`` by ``id``."""
    if not ids:
        return
    placeholders = ",".join(["%s"] * len(ids))
    sql = f"DELETE FROM `{table}` WHERE id IN ({placeholders})"
    try:
        with conn.cursor() as cursor:
            cursor.execute(sql, ids)
            conn.commit()
    except pymysql.MySQLError as e:
        conn.rollback()
        logging.error(f"Error deleting many from {table}: {e}")
        raise e

