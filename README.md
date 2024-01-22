# Jirascope
[![MELPA](https://melpa.org/packages/jirascope-badge.svg)](https://melpa.org/#/jirascope)

Emacs package for Jira Cloud integration.

## Usage
In your Emacs configuration, after `(require 'jirascope)`, setup the connection to a Jira Cloud
instance via `jirascope-setup`. You need to provide the URL, login email and an
[API Token](https://support.atlassian.com/atlassian-account/docs/manage-api-tokens-for-your-atlassian-account/).

Example:
```
el
(defvar jirascope-url "https://example.atlassian.net")
(defvar jirascope-login "me@example.org")
(defvar jirascope-api-token "0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF")

(require 'jirascope)
(jirascope-setup jirascope-url jirascope-login jirascope-api-token)
```
