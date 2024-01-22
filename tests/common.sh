#!/bin/bash

# Run an Emacs command after setting up Jirascope
# Pass the STDIN to the command
run_jirascope_command() {
    local command=$@

    emacs --batch --eval "(add-to-list 'load-path \"../jirascope\")" \
        --eval "(add-to-list 'load-path \"./with-simulated-input\")" \
        --eval "(require 'with-simulated-input)" \
        --eval "(require 'jirascope)" \
        --eval "(jirascope-setup \"$JIRA_URL\" \"$JIRA_USERNAME\" \"$JIRA_API_TOKEN\")" \
        --eval "$command"
}

query_jira_api() {
    local query=$1
    local basic_auth=$(printf "$JIRA_USERNAME:$JIRA_API_TOKEN" | basenc --base64url | tr -d '\n')
    curl -X GET -H "Content-Type: application/json" -H "Authorization: Basic $basic_auth" "$JIRA_URL$query"
}
