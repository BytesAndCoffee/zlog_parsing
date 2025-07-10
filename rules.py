from typing import Any
import logging
from psconnect import fetch_user, Connection

Rule = dict[str, Any]
Row = dict[str, Any]


def validate_rule(rule: Rule) -> bool:
    required_keys = {"type", "match"}
    if not all(k in rule for k in required_keys):
        logging.debug(f"Rule missing required keys: {rule}")
        return False

    if rule["type"] not in ["substring", "pm"]:
        logging.debug(f"Unsupported rule type: {rule['type']}")
        return False

    if not isinstance(rule["match"], str):
        logging.debug(f"Rule 'match' is not a string: {rule['match']}")
        return False

    if "case_sensitive" in rule and not isinstance(rule["case_sensitive"], bool):
        logging.debug(f"Rule 'case_sensitive' is not a bool: {rule['case_sensitive']}")
        return False

    for cond_type in ("only_if", "not_if"):
        if cond_type in rule and not isinstance(rule[cond_type], dict):
            logging.debug(f"Rule '{cond_type}' is not a dict: {rule[cond_type]}")
            return False

    return True


def validate_rules(rules: list[Rule]) -> bool:
    return all(validate_rule(rule) for rule in rules)


def match_rule(rule: Rule, row: Row) -> bool:
    msg = row.get("message", "")
    window = row.get("window", "")
    sender = row.get("nick", "")
    case_sensitive = rule.get("case_sensitive", False)
    match_val = rule["match"]

    msg_cmp = msg if case_sensitive else msg.lower()
    match_cmp = match_val if case_sensitive else match_val.lower()

    if rule["type"] == "pm":
        if window == sender and not window.startswith('#'):
            logging.debug(f"PM rule matched for window {window} and sender {sender}")
            return True
        else:
            return False

    not_if = rule.get("not_if", {})
    if not_if:
        matches_all = True
        for key, val in not_if.items():
            if key == "contains":
                if val in msg:
                    logging.debug(f"Rule {rule} skipped due to not_if contains match")
                    return False
            elif row.get(key) == val:
                logging.debug(f"Rule {rule} skipped due to not_if key={key} val={val}")
                return False

    only_if = rule.get("only_if", {})
    for key, val in only_if.items():
        if key == "contains":
            if val not in msg:
                logging.debug(f"Rule {rule} failed only_if contains: {val}")
                return False
        elif row.get(key) != val:
            logging.debug(f"Rule {rule} failed only_if {key}={val}")
            return False

    if match_cmp not in msg_cmp:
        logging.debug(f"Rule {rule} did not match substring in message")
        return False

    return True


def fetch_rules(conn: Connection, nickname: str) -> list[dict]:
    try:
        user = fetch_user(conn, nickname)
        if not user:
            logging.debug(f"User {nickname} not found while fetching rules.")
            return []

        rules = user.get("hotwords")
        if isinstance(rules, list):
            return rules
        else:
            logging.error(f"Hotwords for {nickname} are not a list: {rules}")
            return []
    except Exception as e:
        logging.error(f"Failed to fetch rules for {nickname}: {e}")
        return []
