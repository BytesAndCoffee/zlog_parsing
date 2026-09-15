"""Small SQL helpers and repositories shared by the workers."""

import logging
from typing import Any, Optional

import pymysql
from pymysql.cursors import Cursor

from zlog_parsing.database.connection import Row
from zlog_parsing.database.schemas import TABLE_SCHEMAS, validate_schema


def insert_into(conn: pymysql.Connection, row: Row, table: str) -> None:
    """Insert a schema-validated row."""
    if not validate_schema(row, table):
        raise ValueError("Invalid schema")
    columns = ", ".join(f"`{column}`" for column in row)
    values = ", ".join(f"%({column})s" for column in row)
    sql = f"INSERT INTO `{table}` ({columns}) VALUES ({values})"
    try:
        with conn.cursor() as cursor:
            cursor.execute(sql, row)
            conn.commit()
    except pymysql.MySQLError as exc:
        conn.rollback()
        logging.error("Error inserting into %s: %s", table, exc)
        raise


def insert_ignore_into(conn: pymysql.Connection, row: Row, table: str) -> bool:
    """Insert a row idempotently and return whether a row was created."""
    if not validate_schema(row, table):
        raise ValueError("Invalid schema")
    columns = ", ".join(f"`{column}`" for column in row)
    values = ", ".join(f"%({column})s" for column in row)
    sql = f"INSERT IGNORE INTO `{table}` ({columns}) VALUES ({values})"
    with conn.cursor() as cursor:
        cursor.execute(sql, row)
        return cursor.rowcount == 1


def replace_into(conn: pymysql.Connection, row: Row, table: str) -> None:
    """Replace a schema-validated row."""
    if not validate_schema(row, table):
        raise ValueError("Invalid schema")
    columns = ", ".join(f"`{column}`" for column in row)
    values = ", ".join(f"%({column})s" for column in row)
    sql = f"REPLACE INTO `{table}` ({columns}) VALUES ({values})"
    try:
        with conn.cursor() as cursor:
            cursor.execute(sql, row)
            conn.commit()
    except pymysql.MySQLError as exc:
        conn.rollback()
        logging.error("Error replacing into %s: %s", table, exc)
        raise


def select_from(
    conn: pymysql.Connection,
    table: str,
    base: int = 28_000_000,
    desc: bool = False,
    limit: int = 1000,
    end: Optional[int] = None,
) -> list[dict[str, Any]]:
    """Select a bounded ID range from a known table."""
    if table not in TABLE_SCHEMAS:
        raise ValueError(f"Unknown table: {table}")
    if limit < 1:
        raise ValueError("limit must be positive")
    end_clause = " AND id <= %s" if end is not None else ""
    sql = (
        f"SELECT * FROM `{table}` WHERE id > %s{end_clause} "
        f"ORDER BY id {'DESC' if desc else 'ASC'} LIMIT %s"
    )
    params = [base]
    if end is not None:
        params.append(end)
    params.append(limit)
    cursor: Cursor | Any
    with conn.cursor() as cursor:
        cursor.execute(sql, params)
        return cursor.fetchall()


def delete_from(conn: pymysql.Connection, table: str, conditions: dict) -> None:
    """Delete rows matching explicit conditions from a known table."""
    if table not in TABLE_SCHEMAS:
        raise ValueError(f"Unknown table: {table}")
    if not conditions:
        raise ValueError(
            "Conditions required for deletion to prevent accidental table wipe."
        )
    where_clause = " AND ".join(f"`{column}` = %s" for column in conditions)
    sql = f"DELETE FROM `{table}` WHERE {where_clause}"
    with conn.cursor() as cursor:
        cursor.execute(sql, list(conditions.values()))


def fetch_users(conn: pymysql.Connection) -> list[str]:
    """Fetch all configured notification recipients."""
    with conn.cursor() as cursor:
        cursor.execute("SELECT nickname FROM users")
        rows: list[dict[str, str]] = cursor.fetchall()
    return [row["nickname"] for row in rows]


def fetch_user(conn: pymysql.Connection, nickname: str) -> Optional[dict]:
    """Fetch one user record by nickname."""
    with conn.cursor() as cursor:
        cursor.execute("SELECT * FROM users WHERE nickname = %s", (nickname,))
        return cursor.fetchone()
