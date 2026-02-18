#!/bin/bash

# Start the zlog_queue.py script in the background
python3 zlog_queue.py &

# Start the parse_logs.py script in the foreground
python3 parse_logs.py
