# Project Setup

These instructions cover setting up the project for local development.

## Prerequisites

- Python 3.11+
- `pip` for installing dependencies

## Installation

1. **Clone the repository**
   ```sh
   git clone https://github.com/BytesAndCoffee/zlog_parsing.git
   cd zlog_parsing
   ```

2. **Create and activate a virtual environment**
   ```sh
   python -m venv venv
   source venv/bin/activate  # On Windows use venv\Scripts\activate
   ```

3. **Install dependencies**
   ```sh
   pip install -r requirements.txt
   ```

## Environment Variables

Copy `.env.example` to `.env` and provide your database information:

```sh
DB_HOST=your_database_host
DB_USERNAME=your_database_username
DB_PASSWORD=your_database_password
DB_NAME=your_database_name
```

## Running the Project

Start the queue and parser scripts:

```sh
python parse_logs.py &
python zlog_queue.py &
```

Alternatively run `main.sh` which launches both scripts.

## Docker

The repository includes a Dockerfile (`main.dockerfile`). Build and run with:

```sh
docker build -f main.dockerfile -t zlog_parsing .
docker run -d zlog_parsing
```
