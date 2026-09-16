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
   pip install -e .
   ```

## Environment Variables

Copy `.env.example` to `.env` and provide your database information:

```sh
DB_HOST=your_database_host
DB_USERNAME=your_database_username
DB_PASSWORD=your_database_password
DB_NAME=your_database_name
TELEGRAM_BOT_TOKEN=your_telegram_bot_token
TELEGRAM_CHAT_ID=your_telegram_chat_id
```

The Telegram values are used only for operational database sleep/wake alerts.
Keep `.env` out of Git and rotate any token that appears in application logs.

The live parser polls an active queue every second. While the queue remains
empty, it doubles that interval up to five seconds to reduce idle database
traffic, then resets to one second as soon as work arrives. Override these
limits with `LIVE_QUEUE_POLL_SECONDS` and `LIVE_QUEUE_MAX_POLL_SECONDS`.

Compose creates the `zlog-state` volume automatically. It stores the outage
marker, recovery timestamp, and durable catch-up jobs independently of MySQL.

## Running the Project

Start the three workers:

```sh
zlog-producer &
zlog-live-parser &
zlog-catchup &
```

Alternatively run `main.sh`, which launches and supervises all three workers.

For production, use:

```sh
docker compose up -d --build
```

## Docker

The repository includes a Dockerfile (`main.dockerfile`). Build and run with:

```sh
docker build -f main.dockerfile -t zlog_parsing .
docker run -d zlog_parsing
```
