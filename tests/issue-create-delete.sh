#!/bin/bash

set -e

source ./common.sh

RANDOM_ISSUE_SUMMARY=$(cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w 32 | head -n 1)

run_jirascope_command "(with-simulated-input \"Placeholder RET Task RET $RANDOM_ISSUE_SUMMARY RET RET\" (jirascope-issue-create))"

# Check if it worked

query_jira_api "/rest/api/3/search" | jq -e ".issues[] | select(.fields.summary == \"$RANDOM_ISSUE_SUMMARY\")" > /dev/null

CREATED_ISSUE_KEY=$(query_jira_api "/rest/api/3/search" | jq -r ".issues[] | select(.fields.summary == \"$RANDOM_ISSUE_SUMMARY\") | .key")

# Clean up

run_jirascope_command "(with-simulated-input \"$CREATED_ISSUE_KEY RET\" (jirascope-issue-delete))"
