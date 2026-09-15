"""Consistent rotating-file logging for worker processes."""

import logging
from logging.handlers import RotatingFileHandler

FORMAT = "%(asctime)s - %(levelname)s - %(message)s"


def build_worker_logger(
    name: str,
    log_file: str,
    *,
    error_file: str | None = None,
) -> logging.Logger:
    """Build a logger and route shared-module messages to the same files."""
    logger = logging.getLogger(name)
    logger.setLevel(logging.INFO)
    logger.propagate = True

    root = logging.getLogger()
    root.setLevel(logging.INFO)
    if any(
        getattr(handler, "_zlog_worker_handler", False) for handler in root.handlers
    ):
        return logger

    formatter = logging.Formatter(FORMAT)
    handler = RotatingFileHandler(log_file, maxBytes=1_000_000, backupCount=10)
    handler.setLevel(logging.INFO)
    handler.setFormatter(formatter)
    handler._zlog_worker_handler = True
    root.addHandler(handler)

    if error_file:
        error_handler = RotatingFileHandler(
            error_file, maxBytes=1_000_000, backupCount=5
        )
        error_handler.setLevel(logging.ERROR)
        error_handler.setFormatter(formatter)
        error_handler._zlog_worker_handler = True
        root.addHandler(error_handler)
    return logger
