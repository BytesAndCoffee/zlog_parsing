"""Durable local recovery state, independent of the sleeping database."""

import sqlite3
from datetime import datetime
from pathlib import Path
from typing import Optional

from settings import STATE_DB_PATH


def connect() -> sqlite3.Connection:
    path = Path(STATE_DB_PATH)
    path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(path, timeout=30)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA journal_mode=WAL")
    conn.executescript("""
        CREATE TABLE IF NOT EXISTS catchup_jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            start_id INTEGER NOT NULL,
            end_id INTEGER NOT NULL,
            next_id INTEGER NOT NULL,
            created_before TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            completed_at TEXT,
            UNIQUE(start_id, end_id)
        );
        CREATE TABLE IF NOT EXISTS recovery_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            created_before TEXT NOT NULL,
            active INTEGER NOT NULL DEFAULT 1
        );
        """)
    return conn


def add_catchup_job(start_id: int, end_id: int, created_before: datetime) -> None:
    with connect() as conn:
        conn.execute(
            """
            INSERT OR IGNORE INTO catchup_jobs
                (start_id, end_id, next_id, created_before, status)
            VALUES (?, ?, ?, ?, 'pending')
            """,
            (start_id, end_id, start_id, created_before.isoformat()),
        )


def get_pending_job() -> Optional[dict]:
    with connect() as conn:
        row = conn.execute("""
            SELECT * FROM catchup_jobs
            WHERE status IN ('pending', 'running')
            ORDER BY id LIMIT 1
            """).fetchone()
    if not row:
        return None
    job = dict(row)
    job["created_before"] = datetime.fromisoformat(job["created_before"])
    return job


def mark_job_running(job_id: int) -> None:
    with connect() as conn:
        conn.execute(
            "UPDATE catchup_jobs SET status = 'running' WHERE id = ?", (job_id,)
        )


def update_job_next_id(job_id: int, next_id: int) -> None:
    with connect() as conn:
        conn.execute(
            "UPDATE catchup_jobs SET next_id = ? WHERE id = ?", (next_id, job_id)
        )


def complete_job(job_id: int) -> None:
    with connect() as conn:
        conn.execute(
            """
            UPDATE catchup_jobs
            SET status = 'complete', completed_at = CURRENT_TIMESTAMP
            WHERE id = ?
            """,
            (job_id,),
        )


def set_recovery_cutoff(created_before: datetime) -> None:
    with connect() as conn:
        conn.execute(
            """
            INSERT INTO recovery_state (id, created_before, active)
            VALUES (1, ?, 1)
            ON CONFLICT(id) DO UPDATE SET
                created_before = excluded.created_before,
                active = 1
            """,
            (created_before.isoformat(),),
        )


def get_recovery_cutoff() -> Optional[datetime]:
    with connect() as conn:
        row = conn.execute(
            "SELECT created_before FROM recovery_state WHERE id = 1 AND active = 1"
        ).fetchone()
    return datetime.fromisoformat(row["created_before"]) if row else None
