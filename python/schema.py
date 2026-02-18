#!/home/michael/.pyenv/shims/python
# schema.py

from typing import Optional
import json
from typing import Any, Dict, Optional
from datetime import datetime
from psconnect import Connection, get_db_connection



def convert_type(mysql_type: str) -> str:
    """
    Converts MySQL data types to equivalent Python type hints.
    """
    mapping = {
        'int': int,
        'varchar': str,
        'text': str,
        'datetime': datetime,
        # Add more mappings as needed
    }
    return mapping.get(mysql_type, 'Any')

def fetch_schema(database: str, table: str, conn: Connection) -> Optional[Dict[str, Any]]:
    """
    Fetches the schema of a specified table from the database.
    """
    query = f"""
    SELECT 
      JSON_OBJECT(
        'meta-schema', JSON_OBJECT('column', JSON_ARRAY('type', 'nullable')),
        'columns', (
          SELECT 
            JSON_ARRAYAGG(
              JSON_OBJECT(
                COLUMN_NAME, JSON_ARRAY(DATA_TYPE, IF(IS_NULLABLE='YES', 'true', 'false'))
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
        cursor.execute(query, (database, table))  # Execute the query with the provided database and table
        result = cursor.fetchone()  # Fetch the result of the query
        if result:
            schema = json.loads(result['complete_table_schema'])  # Load the JSON result into a dictionary
            return schema
        else:
            return None

def print_schema(schema: Dict[str, Any], table_name: str) -> None:
    """
    Prints the schema of a specified table in a formatted manner.
    """
    if schema:
        # Convert schema to the specified format
        converted_columns = []
        for column in schema.get('columns', []):
            for name, details in column.items():
                # Convert MySQL boolean strings to Python booleans and types to Python type names
                python_type = convert_type(details[0])
                is_nullable = 'True' if details[1] == 'true' else 'False'
                # Manually format the column string to match the desired output
                column_str = f'{{"{name}": [{str(python_type)[8:][:-2]}, {is_nullable}]}}'
                converted_columns.append(column_str)
        
        # Manually construct the output string to match the requested format
        columns_str = ',\n            '.join(converted_columns)
        output = f'"{table_name}": {{\n    "meta-schema": {{\n        "column": ["type", "nullable"]\n    }},\n    "columns": [\n            {columns_str}\n    ]\n}}'
        
        print(output)  # Print the formatted schema
    else:
        print(f"No schema found for table {table_name}.")

# Example usage
if __name__ == "__main__":
    conn: Connection = get_db_connection()  # Get a database connection
    # Example call with 'znc' as database and 'pm_table' as table
    schema = fetch_schema('znc', 'pm_table', conn)  # Fetch the schema of the 'pm_table'
    print_schema(schema, 'pm_table')  # Print the schema of the 'pm_table'
