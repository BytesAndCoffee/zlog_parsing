import unittest

from zlog_parsing.rules import match_rule, validate_rule


class RuleTests(unittest.TestCase):
    def test_pm_rule_remains_backward_compatible(self):
        row = {"id": 1, "window": "alice", "nick": "alice", "message": "hello"}

        self.assertTrue(validate_rule({"type": "pm"}))
        self.assertTrue(match_rule({"type": "pm"}, row))

    def test_substring_conditions_and_sender_exclusion(self):
        rule = {
            "type": "substring",
            "match": "Alert",
            "only_if": {"window": "#ops"},
            "not_if": {"contains": "resolved"},
        }
        matching = {
            "id": 2,
            "window": "#ops",
            "nick": "alice",
            "message": "alert: database slow",
        }
        resolved = {**matching, "message": "alert resolved"}
        sender_contains_match = {**matching, "nick": "alert-bot"}

        self.assertTrue(match_rule(rule, matching))
        self.assertFalse(match_rule(rule, resolved))
        self.assertFalse(match_rule(rule, sender_contains_match))

    def test_invalid_rule_type_is_rejected(self):
        self.assertFalse(validate_rule({"type": "regex", "match": ".*"}))


if __name__ == "__main__":
    unittest.main()
