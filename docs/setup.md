# Project Setup

This document provides detailed instructions for setting up the project, including environment variables and dependencies.

## Prerequisites

Before you begin, ensure you have met the following requirements:
- You have installed Python >= 3.11
- You have installed `pip`, the Python package installer

## Installation

1. **Clone the repository:**
   ```sh
   git clone https://github.com/BytesAndCoffee/zlog_parsing.git
   cd zlog_parsing
   ```

2. **Create a virtual environment:**
   ```sh
   python -m venv venv
   source venv/bin/activate  # On Windows use `venv\Scripts\activate`
   ```

3. **Install the dependencies:**
   ```sh
   pip install -r requirements.txt
   ```

## Environment Variables

The project uses environment variables configured in a `.env` file for database connection. Create a `.env` file in the root directory of the project with the following content:

```sh
DB_HOST=your_database_host
DB_USERNAME=your_database_username
DB_PASSWORD=your_database_password
DB_NAME=your_database_name
```

Replace `your_database_host`, `your_database_username`, `your_database_password`, and `your_database_name` with your actual database credentials.

## Running the Project

After setting up the environment variables and installing the dependencies, you can run the main scripts of the project:

```sh
python parse_logs.py &
python zlog_queue.py &
```

This will start the log parsing and queue management processes.
