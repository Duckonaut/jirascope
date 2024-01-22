#!/bin/bash

set -e

source ./common.sh

RANDOM_ISSUE_SUMMARY=$(cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w 32 | head -n 1)

run_jirascope_command "(with-simulated-input \"Placeholder RET Task RET $RANDOM_ISSUE_SUMMARY RET RET\" (jirascope-issue-create))"

echo "Issue created"

# Check if it worked
FOUND_ISSUE=$(query_jira_api "/rest/api/3/search" | jq -e ".issues[] | select(.fields.summary == \"$RANDOM_ISSUE_SUMMARY\")")

if [ -z "$FOUND_ISSUE" ]; then
  echo "Failed to create issue"
  exit 1
fi

echo "Issue found"

CREATED_ISSUE_KEY=$(echo "$FOUND_ISSUE" | jq -r ".key")

# Clean up

run_jirascope_command "(with-simulated-input \"$CREATED_ISSUE_KEY RET\" (jirascope-issue-delete))"

echo "Issue deleted"

FOUND_ISSUE_COUNT=$(query_jira_api "/rest/api/3/search" | jq -e "[.issues[].fields.summary | select(. == \"$CREATED_ISSUE_KEY\")] | length")

if [ "$FOUND_ISSUE_COUNT" -ne "0" ]; then
  echo "Failed to delete issue"
  exit 1
fi

echo "Issue confirmed deleted"
