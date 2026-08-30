"""Shared rule loading and idempotent notification routing."""

import json
import logging
from typing import Any

from psconnect import Connection, Row, fetch_users, insert_ignore_into
from rules import fetch_rules, match_rule, validate_rules


def load_user_rules(conn: Connection) -> dict[str, list[dict[str, Any]]]:
    loaded: dict[str, list[dict[str, Any]]] = {}
    for user in fetch_users(conn):
        rules = fetch_rules(conn, user)
        logging.debug("Rules loaded for %s: %s", user, json.dumps(rules))
        if validate_rules(rules):
            loaded[user] = rules
        else:
            logging.warning("Rules for %s failed validation", user)
    return loaded


def route_log(
    conn: Connection,
    user_rules: dict[str, list[dict[str, Any]]],
    log: Row,
) -> bool:
    """Route a matching log idempotently; return whether any rule matched."""
    if log["type"] not in ("msg", "action"):
        return False

    matched = False
    for recipient, rules in user_rules.items():
        if not any(match_rule(rule, log) for rule in rules):
            continue
        matched = True
        row = {
            "id": log["id"],
            "user": log["user"],
            "network": log["network"],
            "window": log["window"],
            "type": log["type"],
            "nick": log["nick"],
            "message": log["message"],
            "recipient": recipient,
        }
        insert_ignore_into(conn, row, "event_log")
        insert_ignore_into(conn, row, "push")
    return matched
