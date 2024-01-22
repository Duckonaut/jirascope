#!/bin/bash

set -e

source ./common.sh

RANDOM_PROJECT_KEY=$(cat /dev/urandom | tr -dc 'A-Z' | fold -w 4 | head -n 1)
RANDOM_PROJECT_NAME=$(cat /dev/urandom | tr -dc 'a-zA-Z' | fold -w 32 | head -n 1)
RANDOM_PROJECT_DESCRIPTION=$(cat /dev/urandom | tr -dc 'a-zA-Z' | fold -w 32 | head -n 1)
PROJECT_URL="https://example.com"
PROJECT_LEAD=$(query_jira_api "/rest/api/3/user/search?query=$JIRA_USERNAME" | jq -r ".[0].displayName")
PROJECT_LEAD=$(echo "$PROJECT_LEAD" | sed 's/ / SPC /g')
PROJECT_TYPE="software"
ASSIGNEE_TYPE="Unassigned"
TEMPLATE_TYPE="Basic"

PROGRAM="(with-simulated-input \"$RANDOM_PROJECT_KEY RET $RANDOM_PROJECT_NAME RET $RANDOM_PROJECT_DESCRIPTION RET $PROJECT_URL RET $PROJECT_LEAD RET $PROJECT_TYPE RET $ASSIGNEE_TYPE RET $TEMPLATE_TYPE RET\" (jirascope-project-create))"

run_jirascope_command "$PROGRAM"
echo "Project created"

# Check if it worked
FOUND_PROJECT=$(query_jira_api "/rest/api/3/project/$RANDOM_PROJECT_KEY")
if [ -z "$FOUND_PROJECT" ]; then
  echo "Failed to create project"
  exit 1
fi

echo "Project found"

# Clean up

run_jirascope_command "(with-simulated-input \"$RANDOM_PROJECT_KEY RET\" (jirascope-project-delete))"

echo "Project deleted"

PROJECT_COUNT=$(query_jira_api "/rest/api/3/project/search?maxResults=1000&startAt=0" | jq -r "[.values[].key | select(. == \"$RANDOM_PROJECT_KEY\")] | length")

if [ "$PROJECT_COUNT" -ne "0" ]; then
  echo "Failed to delete project"
  exit 1
fi

echo "Project confirmed deleted"
