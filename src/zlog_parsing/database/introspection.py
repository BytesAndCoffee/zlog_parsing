"""Development utility for inspecting MySQL table schemas."""

import json
from datetime import datetime
from typing import Any, Optional

from zlog_parsing.database import Connection, get_db_connection


def convert_type(mysql_type: str) -> str:
    """
    Converts MySQL data types to equivalent Python type hints.
    """
    mapping = {
        "int": int,
        "varchar": str,
        "text": str,
        "datetime": datetime,
        # Add more mappings as needed
    }
    return mapping.get(mysql_type, "Any")


def fetch_schema(
    database: str, table: str, conn: Connection
) -> Optional[dict[str, Any]]:
    """Fetch a table schema from MySQL's information schema."""
    query = """
    SELECT
      JSON_OBJECT(
        'meta-schema', JSON_OBJECT('column', JSON_ARRAY('type', 'nullable')),
        'columns', (
          SELECT
            JSON_ARRAYAGG(
              JSON_OBJECT(
                COLUMN_NAME,
                JSON_ARRAY(DATA_TYPE, IF(IS_NULLABLE='YES', 'true', 'false'))
              )
            )
          FROM INFORMATION_SCHEMA.COLUMNS
          WHERE TABLE_SCHEMA = %s
            AND TABLE_NAME = %s
        )
      ) AS complete_table_schema
    FROM DUAL;
    """
    with conn.cursor() as cursor:
        cursor.execute(query, (database, table))
        result = cursor.fetchone()
    return json.loads(result["complete_table_schema"]) if result else None


def print_schema(schema: dict[str, Any], table_name: str) -> None:
    """Print a table schema in the runtime-schema source format."""
    if schema:
        converted_columns = []
        for column in schema.get("columns", []):
            for name, details in column.items():
                python_type = convert_type(details[0])
                is_nullable = "True" if details[1] == "true" else "False"
                type_name = (
                    python_type.__name__
                    if isinstance(python_type, type)
                    else python_type
                )
                column_str = f'{{"{name}": [{type_name}, {is_nullable}]}}'
                converted_columns.append(column_str)

        columns_str = ",\n            ".join(converted_columns)
        output = (
            f'"{table_name}": {{\n'
            '    "meta-schema": {\n'
            '        "column": ["type", "nullable"]\n'
            "    },\n"
            '    "columns": [\n'
            f"            {columns_str}\n"
            "    ]\n"
            "}"
        )
        print(output)
    else:
        print(f"No schema found for table {table_name}.")


def main() -> None:
    """Print the configured database's pm_table schema."""
    conn: Connection = get_db_connection()
    try:
        schema = fetch_schema("znc", "pm_table", conn)
        print_schema(schema, "pm_table")
    finally:
        conn.close()


if __name__ == "__main__":
    main()
