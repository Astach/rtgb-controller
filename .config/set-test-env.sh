#!/bin/bash

# Exit with 1 if NEXTEST_ENV isn't defined.
if [ -z "$NEXTEST_ENV" ]; then
	exit 1
fi
# Write out an environment variable to $NEXTEST_ENV.
echo "DATABASE_URL=postgres://$(whoami):localpassword@127.0.0.1/rtgb_scheduler?sslmode=disable" >>"$NEXTEST_ENV"
