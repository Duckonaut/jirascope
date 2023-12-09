# Jiroscope
Emacs package for Jira Cloud integration.

## Usage
In your Emacs configuration, after `(require 'jiroscope)`, setup the connection to a Jira Cloud
instance via `jiroscope-setup`. You need to provide the URL, login email and an
[API Token](https://support.atlassian.com/atlassian-account/docs/manage-api-tokens-for-your-atlassian-account/).

Example:
```
el
(defvar jiroscope-url "https://example.atlassian.net")
(defvar jiroscope-login "me@example.org")
(defvar jiroscope-api-token "0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF")

(require 'jiroscope)
(jiroscope-setup jiroscope-url jiroscope-login jiroscope-api-token)
```
