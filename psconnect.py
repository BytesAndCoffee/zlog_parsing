#!/home/michael/.pyenv/shims/python
# psconnect.py

import os, pymysql, pymysql.cursors
from typing import Optional
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

Row = dict[str, str | int]
logging.basicConfig(level=logging.ERROR, filename='error.log', filemode='a', 
                    format='%(asctime)s - %(levelname)s - %(message)s')

def get_db_connection() -> Connection:
    try:
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
        logging.error(f"Error connecting to the database: {e}")
        raise e
        

def validate_schema(row: Row, table: str) -> bool:
    schema = table_schemas[table]
    for column_spec in schema["columns"]:
        for col, specs in column_spec.items():
            col_type, nullable = specs
            if col not in row:
                if not nullable:
                    logging.error(f"Column {col} is missing from the row")
                    return False
                else:
                    continue  # It's okay for nullable columns to be missing
            if row[col] is None and not nullable:
                logging.error(f"Column {col} cannot be null")
                return False
            if row[col] is not None and not isinstance(row[col], col_type):
                logging.error(f"Column {col} must be of type {col_type.__name__}")
                return False
    return True

def insert_into(conn: pymysql.Connection, row: Row, table: str) -> None:
    if not validate_schema(row, table):
        raise ValueError("Invalid schema")
    cols = ', '.join(f'`{col}`' for col in row.keys())
    vals = ', '.join(f'%({col})s' for col in row.keys())
    sql = f'INSERT INTO `{table}` ({cols}) VALUES ({vals})'
    try:
        with conn.cursor() as cursor:
            cursor.execute(sql, row)
            conn.commit()
    except pymysql.MySQLError as e:
        conn.rollback()
        logging.error(f"Error inserting into {table}: {e}")
        raise e


def replace_into(conn: pymysql.Connection, row: Row, table: str) -> None:
    if not validate_schema(row, table):
        raise ValueError("Invalid schema")
    cols = ', '.join(f'`{col}`' for col in row.keys())
    vals = ', '.join(f'%({col})s' for col in row.keys())
    sql = f'REPLACE INTO `{table}` ({cols}) VALUES ({vals})'
    try:
        with conn.cursor() as cursor:
            cursor.execute(sql, row)
            conn.commit()
    except pymysql.MySQLError as e:
        conn.rollback()
        logging.error(f"Error replacing into {table}: {e}")


def select_from(conn: pymysql.Connection, table: str, base: int = 28000000, desc: bool = False) -> Optional[list[dict]]:
    try:
        with conn.cursor() as cursor:
            cursor.execute(f"SELECT * FROM {table} WHERE id > {base} ORDER BY id {'DESC' if desc else 'ASC'}")
            return cursor.fetchall()
    except pymysql.MySQLError as e:
        logging.error(f"Error selecting from {table}: {e}")
        return None


def delete_from(conn: pymysql.Connection, table: str, conditions: dict) -> None:
    if not conditions:
        logging.error("Conditions required for deletion to prevent accidental table wipe.")
        raise ValueError("Conditions required for deletion to prevent accidental table wipe.")
    
    where_clause_parts = []
    params = []
    for column, value in conditions.items():
        where_clause_parts.append(f"`{column}` = %s")
        params.append(value)
    
    where_clause = " AND ".join(where_clause_parts)
    
    sql = f"DELETE FROM `{table}` WHERE {where_clause}"
    
    try:
        with conn.cursor() as cursor:
            cursor.execute(sql, params)
            conn.commit()
    except pymysql.MySQLError as e:
        conn.rollback()
        logging.error(f"Error deleting from {table}: {e}")