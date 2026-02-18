# Filtering Rules

This document describes the JSON structure used to define filtering rules for
log entries. Each user stores these rules in the `users.hotwords` column. The
column is defined as a JSON field and **must** contain a JSON array of rule
objects, even when a user has only a single rule.

```
CREATE TABLE `users` (
  `nickname` VARCHAR(64) NOT NULL,
  `telegram_chat_id` BIGINT DEFAULT NULL,
  `hotwords` JSON DEFAULT NULL,
  PRIMARY KEY (`nickname`)
);
```

Example with a single rule:

```json
[
  {"type": "pm"}
]
```

## Rule Object

A rule is a JSON object with the following common fields:

- `type` – either `substring` or `pm`.
- `case_sensitive` – optional boolean, defaults to `false` when omitted.
- `only_if` – optional object of additional conditions that **must** match.
- `not_if` – optional object of conditions that suppress the rule when all match.

### Substring Rules

```json
{
  "type": "substring",
  "match": "hello",
  "case_sensitive": false,
  "only_if": {"window": "#support"},
  "not_if": {"nick": "bot"}
}
```

- `match` – substring to look for in the log `message`.
- `only_if` may contain field/value pairs or `contains` to require a substring in the message.
- `not_if` works the same way but disables the rule when satisfied.

### PM Rules

```json
{"type": "pm"}
```

A PM rule triggers when a log entry is a private message (`window` equals `nick` and is not a channel). No other fields are required.

## Evaluation Logic

`parse_logs.py` obtains the rule list for each user and evaluates them using `rules.match_rule`. Rules are processed in the order they appear, and the first match causes the log entry to be queued for that user.

