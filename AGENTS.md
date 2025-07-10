# Project Agents.md Guide for OpenAI Codex

This Agents.md file provides structured guidance for OpenAI Codex and similar AI agents working with the `zlog_parsing` system — a rule-driven log analysis engine for ZNC-integrated infrastructure.

## Project Structure for OpenAI Codex Navigation

- `parse_logs.py`: Main entry point for log processing
  - Codex should inspect this to understand how logs are matched to user-defined rules and pushed into downstream queues
- `rules.py`: Declarative rule engine logic
  - Codex may extend this with new rule types, but must maintain compatibility with JSON rule blobs stored in the `users` table
- `zlog_queue.py`: Handles message polling and log movement from `logs` to `logs_queue`
  - Codex may optimize or modularize this flow
- `psconnect.py`: Manages database connections and SQL helpers
  - Codex must preserve connection pooling and transaction safety
- `schema.py`: Embedded table schemas
  - Codex may extend this when new tables are introduced but should update `zlog_schema.sql` accordingly
- `zlog_schema.sql`: Authoritative SQL schema for Zlog-related tables
  - Codex should mirror all schema edits here
- `docs/`: Markdown documentation (setup, usage, modules)
  - Codex may contribute improvements, diagrams, or explanations

## Coding Conventions for OpenAI Codex

### General Conventions

- Use Python 3.10+ syntax
- Follow PEP8 and autoformat using `black`
- Functions must use explicit naming (`evaluate_rule`, `insert_into`)
- Avoid tight coupling between rule logic and DB schema

### Rule Logic Guidelines

- Rules are stored in JSON format per user in the `users.hotwords` column
- Codex may create new rule types (e.g. regex, negation, or AND-chains)
- All new rule types must be backward-compatible
- Output of rule evaluation must be easily serializable for DB insert

### Message Routing and Database Writing

- `parse_logs.py` evaluates logs against rules, and inserts matched entries into the `push` table
- Codex must ensure:
  - Logs are only inserted once
  - Duplicates are not reprocessed
  - Recipient routing remains based on matched user context

## Database Conventions for OpenAI Codex

Codex should ensure:

- All new table definitions or changes reflect in `zlog_schema.sql`
- Changes are non-breaking to existing deployments
- All writes (inserts, updates) are idempotent and logged
- Codex does not hardcode Telegram-specific routing here

## Testing and Execution Guidelines

Codex must ensure the following checks are valid:

```bash
# Dry run log parsing
python parse_logs.py --dry-run

# Validate rule format
python -m rules validate

# Run unit tests (if added later)
pytest tests/
```

Unit test stubs may be added in future `/tests` directory.

## Pull Request Guidelines for OpenAI Codex

When OpenAI Codex generates a PR, ensure it:

1. Contains only changes relevant to `zlog_parsing-master`
2. Includes docstring updates for any modified function
3. Reflects schema changes in `zlog_schema.sql` and/or `schema.py`
4. Includes example usage in `docs/usage.md` if applicable

## Programmatic Checks for Codex Contributions

Before merging any Codex-generated work, run:

```bash
# Check style and formatting
black .

# Check for unused imports and linting
flake8 .

```

All checks must pass to maintain stability of the parsing system. Codex is expected to prioritize performance, clarity, and resilience in all enhancements.

---
This `AGENTS.md` exists to help OpenAI Codex understand the structure, responsibilities, and evolution strategy of the Zlog parsing engine.
