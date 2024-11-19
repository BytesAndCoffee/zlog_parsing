# Usage

This document provides detailed instructions for using the project, including running scripts and expected behavior.

## Running the Scripts

The main scripts in this project are:

- `psconnect.py`: This script provides the connection and interaction with MySQL (Specifically Planetscale in this project)
- `parse_logs.py`: This script parses logs and updates the database accordingly.
- `zlog_queue.py`: This script manages a queue of logs, marking them as processed and copying new logs.

## Environment Variables

The project uses environment variables configured in a `.env` file for database connection. The `.env` file should be in the following format:

```
DB_HOST=host
DB_USERNAME=username
DB_PASSWORD=password
DB_NAME=database
```

## Dependencies

The project dependencies are listed in the `requirements.txt` file. You can install them using the following command:

```sh
pip install -r requirements.txt
```

## Running the Scripts

You can run the scripts as follows:

```sh
python parse_logs.py &
python zlog_queue.py &
```

## Expected Behavior

- `psconnect.py`: Establishes a connection to the MySQL database and provides functions for interacting with the database.
- `parse_logs.py`: Parses logs from the `logs_queue` table and updates the `push` and `event_log` tables accordingly.
- `zlog_queue.py`: Manages a queue of logs, marking them as processed and copying new logs from the `logs` table to the `logs_queue` table.
