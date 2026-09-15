#!/bin/bash

set -u

queue_pid=""
parser_pid=""
catchup_pid=""

shutdown() {
    trap - EXIT INT TERM
    [ -z "$queue_pid" ] || kill "$queue_pid" 2>/dev/null || true
    [ -z "$parser_pid" ] || kill "$parser_pid" 2>/dev/null || true
    [ -z "$catchup_pid" ] || kill "$catchup_pid" 2>/dev/null || true
    wait 2>/dev/null || true
}

trap shutdown EXIT INT TERM

python3 zlog_queue.py &
queue_pid=$!

python3 parse_logs.py &
parser_pid=$!

python3 catchup_logs.py &
catchup_pid=$!

# If any worker stops, exit the container so Docker can restart the full set
# with fresh database connections.
wait -n "$queue_pid" "$parser_pid" "$catchup_pid"
exit $?
