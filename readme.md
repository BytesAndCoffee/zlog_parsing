# zlog parsing

This is a Python project that provides utilities for parsing logs and managing a queue of logs. It also includes functionality for connecting to a database and manipulating data.

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes.

### Requirements

- Python >= 3.11
- PyMySQL
- dotenv

## Usage

The main scripts in this project are:
- `psconnect.py`: This script provides the connection and interaction with MySQL (Specifically Planetscale in this project)
- `parse_logs.py`: This script parses logs and updates the database accordingly.
- `zlog_queue.py`: This script manages a queue of logs, marking them as processed and copying new logs.
- `.env`: An environment variables file used to configure PyMySQL, in the following format:
    ```.env
    DB_HOST=host
    DB_USERNAME=username
    DB_PASSWORD=password
    DB_NAME=databse
    ```

You can run these scripts as follows:

```sh
python parse_logs.py &
python zlog_queue.py &
```

## Documentation

Comprehensive documentation for the project is available in the `docs` directory. The documentation includes detailed instructions for setup, usage, and descriptions of each module and function.

- [Setup Instructions](docs/setup.md)
- [Usage Instructions](docs/usage.md)
- [Modules Documentation](docs/modules.md)

## License

This project is licensed under the MIT License
