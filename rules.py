# rules.py

from typing import Any
import logging

from psconnect import fetch_user, Connection

Rule = dict[str, Any]
Row = dict[str, Any]


def validate_rule(rule: Rule) -> bool:
    """
    Validates the structure of a single hotword rule dict.
    """
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
    """
    Validates a list of rule dicts using validate_rule().
    """
    return all(validate_rule(rule) for rule in rules)


def match_rule(rule: Rule, row: Row) -> bool:
    """
    Evaluates whether a log row matches a given rule.
    """
    msg = row.get("message", "")
    window = row.get("window", "")
    sender = row.get("nick", "")
    case_sensitive = rule.get("case_sensitive", False)
    match_val = rule["match"]

    msg_cmp = msg if case_sensitive else msg.lower()
    match_cmp = match_val if case_sensitive else match_val.lower()

    if rule["type"] == "pm":
        # PM rules only match if the window is a PM and the sender matches
        if window == sender and not window.startswith('#'):
            return True
        else:
            return False

    # Check 'not_if' conditions first
    not_if = rule.get("not_if", {})
    if not_if:
        for key, val in rule.get("not_if", {}).items():
            if key == "contains" and val in msg:
                return False
            if key != "contains" and row.get(key) == val:
                return False

    # Then check 'only_if' conditions
    for key, val in rule.get("only_if", {}).items():
        if key == "contains" and val in msg:
            return True
        if row.get(key) != val:
            return False

    # Finally evaluate match
    if match_cmp not in msg_cmp:
        return False
    logging.debug(f"Rule matched: {rule} for log {row['id']}")
    return True


def fetch_rules(conn: Connection, nickname: str) -> list[dict]:
    """
    Fetches the list of hotword rules for a given user by calling fetch_user().
    Assumes 'hotwords' is a JSON column containing a list of rule dicts.
    Returns an empty list if the user is not found or an error occurs.
    """
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
