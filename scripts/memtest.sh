#!/bin/bash
REPORT_FILE=/tmp/valgrind-out.txt
set -e

valgrind --leak-check=full \
         --show-leak-kinds=all \
         --track-origins=yes \
         --verbose \
         --log-file=$REPORT_FILE \
		 $@

echo "done. Opening report: $REPORT_FILE"
read
less $REPORT_FILE
