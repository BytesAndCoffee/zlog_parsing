# rules.py

from typing import Any
import logging
import json

from psconnect import fetch_user, Connection

Rule = dict[str, Any]
Row = dict[str, Any]

def validate_rule(rule: Rule) -> bool:
    """
    Validates the structure of a single hotword rule dict.
    """
    if "type" not in rule:
        logging.debug(f"Rule missing 'type': {rule}")
        return False

    if rule["type"] == "substring":
        if "match" not in rule or not isinstance(rule["match"], str):
            logging.debug(f"Substring rule missing or invalid 'match': {rule}")
            return False
    elif rule["type"] == "pm":
        # pm rules are valid without match
        return True
    else:
        logging.debug(f"Unsupported rule type: {rule['type']}")
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
    try:
        msg = row.get("message", "")
        window = row.get("window", "")
        sender = row.get("nick", "")

        logging.debug(f"Evaluating rule on log {row.get('id')}: {rule}")

        # PM rule: window is the sender and not a channel
        if rule["type"] == "pm":
            is_pm = window == sender and not window.startswith("#")
            logging.debug(f"PM rule evaluation: window={window}, sender={sender}, result={is_pm}")
            return is_pm

        # Substring rule matching
        match_val = rule.get("match", "")
        case_sensitive = rule.get("case_sensitive", False)

        msg_cmp = msg if case_sensitive else msg.lower()
        match_cmp = match_val if case_sensitive else match_val.lower()
        nick_cmp = sender if case_sensitive else sender.lower()

        # 'not_if' logic - ALL conditions must be satisfied to suppress
        not_if = rule.get("not_if", {})
        if not_if:
            suppress = True
            for key, val in not_if.items():
                if key == "contains":
                    condition = val if case_sensitive else val.lower()
                    if condition not in msg_cmp:
                        suppress = False
                        break
                else:
                    field = row.get(key, "")
                    field_cmp = field if case_sensitive else field.lower()
                    val_cmp = val if case_sensitive else val.lower()
                    if field_cmp != val_cmp:
                        suppress = False
                        break
            if suppress:
                logging.debug(f"Rule suppressed by not_if conditions: {not_if}")
                return False

        # 'only_if' logic - ALL must match
        for key, val in rule.get("only_if", {}).items():
            if key == "contains":
                condition = val if case_sensitive else val.lower()
                if condition not in msg_cmp:
                    logging.debug(f"Rule skipped by only_if.contains (not found): {val}")
                    return False
            else:
                field = row.get(key, "")
                field_cmp = field if case_sensitive else field.lower()
                val_cmp = val if case_sensitive else val.lower()
                if field_cmp != val_cmp:
                    logging.debug(f"Rule skipped by only_if[{key}]: {val_cmp} != {field_cmp}")
                    return False

        # Final substring match
        if match_cmp in msg_cmp and not match_cmp in nick_cmp:
            logging.debug(f"Rule matched log {row.get('id')} with match='{match_val}'")
            return True
        else:
            logging.debug(f"Substring '{match_val}' not found in message")
            return False

    except Exception as e:
        logging.error(f"Error evaluating rule {rule} on row {row.get('id')}: {e}")
        return False


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
        if isinstance(rules, str):
            try:
                rules = json.loads(rules)
            except json.JSONDecodeError as e:
                logging.error(f"Hotwords for {nickname} could not be decoded: {e}")
                return []

        if isinstance(rules, list):
            return rules
        else:
            logging.error(f"Hotwords for {nickname} are not a list after parsing: {rules}")
            return []
    except Exception as e:
        logging.error(f"Failed to fetch rules for {nickname}: {e}")
        return []
