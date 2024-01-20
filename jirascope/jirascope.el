;;; jirascope.el --- A Jira client -*- lexical-binding: t; coding: utf-8 -*-

;; SPDX-License-Identifier: MIT OR Apache-2.0
;; Author: Stanisław Zagórowski <duckonaut@gmail.com>
;; Version: 0.2.1
;; Keywords: tools
;; URL: https://github.com/Duckonaut/jirascope
;; Package-Requires: ((emacs "25.1"))

;;; Commentary:
;; Jirascope is a package integrating Jira Cloud into Emacs,
;; allowing you to operate on projects and issues through either
;; prompt-based interactive commands, or special buffers.

;; Core functionality is implemented in Rust. The Rust portion is
;; distributed via github (automatically downloaded as needed)
;; and loaded as a dynamic Emacs module

;;; Code:

(unless (functionp 'module-load)
  (error "Dynamic module feature not available, please compile Emacs --with-modules option turned on"))

(defconst jirascope--dyn-version "0.2.1"
  "Required version of the dynamic module `jirascope-dyn'.")

(defconst jirascope--first-time-user-error "First time using jirascope/binary is missing. Please run `jirascope-install' to install it"
  "Error message shown to the user when the dynamic module is not installed.")

(defgroup jirascope nil
  "Core jirascope APIs."
  :group 'jirascope)

(defconst jirascope--dyn-dir (file-name-directory (or (locate-library "jirascope.el") ""))
  "The directory where the library `jirascope' is located.")

(defcustom jirascope-dyn-dir jirascope--dyn-dir
  "The directory that `jirascope-dyn' module is resided.
This should be set before `jirascope' is loaded.

Example setting:
\(setq jirascope-dyn-dir (expand-file-name \"jirascope/\" user-emacs-directory))"
  :group 'jirascope
  :type 'directory)

(require 'jirascope-dyn-get)

(defun jirascope-install ()
  "Install the dynamic module `jirascope-dyn'."
  (interactive)
  (jirascope-dyn-get-install jirascope--dyn-version))

(if (jirascope-dyn-get-installed)
  (require 'jirascope-dyn))

;; declare functions from the dynamic module to satisfy byte-compiler
(declare-function jirascope-dyn-setup "ext:jirascope-dyn")
(declare-function jirascope-dyn-issue-create-interactive "ext:jirascope-dyn")
(declare-function jirascope-dyn-issue-display-interactive "ext:jirascope-dyn")
(declare-function jirascope-dyn-issue-edit-interactive "ext:jirascope-dyn")
(declare-function jirascope-dyn-issue-edit-graphical-interactive "ext:jirascope-dyn")
(declare-function jirascope-dyn-issue-edit-graphical-finish "ext:jirascope-dyn")
(declare-function jirascope-dyn-issue-delete-interactive "ext:jirascope-dyn")
(declare-function jirascope-dyn-issue-transition-interactive "ext:jirascope-dyn")
(declare-function jirascope-dyn-project-create-interactive "ext:jirascope-dyn")
(declare-function jirascope-dyn-project-edit-interactive "ext:jirascope-dyn")
(declare-function jirascope-dyn-project-edit-graphical-interactive "ext:jirascope-dyn")
(declare-function jirascope-dyn-project-edit-graphical-finish "ext:jirascope-dyn")
(declare-function jirascope-dyn-project-delete-interactive "ext:jirascope-dyn")
(declare-function jirascope-dyn-state-open "ext:jirascope-dyn")

(defun jirascope-setup (url login api_token)
  "Setup Jirascope with the given cloud URL, LOGIN and API_TOKEN."
  (if (jirascope-dyn-get-installed)
    (jirascope-dyn-setup url login api_token)
    (user-error jirascope--first-time-user-error)))

;; add bindings for interactive use
(defun jirascope-issue-create ()
  "Create a new issue."
  (interactive)
  (if (jirascope-dyn-get-installed)
    (jirascope-dyn-issue-create-interactive)
    (user-error jirascope--first-time-user-error)))

(defun jirascope-issue-display ()
  "Display an issue."
  (interactive)
  (if (jirascope-dyn-get-installed)
    (jirascope-dyn-issue-display-interactive)
    (user-error jirascope--first-time-user-error)))

(defun jirascope-issue-edit ()
  "Edit an issue via prompt."
  (interactive)
  (if (jirascope-dyn-get-installed)
    (jirascope-dyn-issue-edit-interactive)
    (user-error jirascope--first-time-user-error)))

(defun jirascope-issue-edit-graphical ()
  "Edit an issue in a buffer."
  (interactive)
  (if (jirascope-dyn-get-installed)
    (jirascope-dyn-issue-edit-graphical-interactive)
    (user-error jirascope--first-time-user-error)))

(defun jirascope-issue-edit-finish ()
  "Finish editing an issue in a buffer and send it to the server."
  (interactive)
  (if (jirascope-dyn-get-installed)
    (jirascope-dyn-issue-edit-graphical-finish)
    (user-error jirascope--first-time-user-error)))

(defun jirascope-issue-delete ()
  "Delete an issue."
  (interactive)
  (if (jirascope-dyn-get-installed)
    (jirascope-dyn-issue-delete-interactive)
    (user-error jirascope--first-time-user-error)))

(defun jirascope-issue-transition ()
  "Transition an issue."
  (interactive)
  (if (jirascope-dyn-get-installed)
    (jirascope-dyn-issue-transition-interactive)
    (user-error jirascope--first-time-user-error)))

(defun jirascope-project-create ()
  "Create a project via prompt."
  (interactive)
  (if (jirascope-dyn-get-installed)
    (jirascope-dyn-project-create-interactive)
    (user-error jirascope--first-time-user-error)))

(defun jirascope-project-edit ()
  "Edit a project via prompt."
  (interactive)
  (if (jirascope-dyn-get-installed)
    (jirascope-dyn-project-edit-interactive)
    (user-error jirascope--first-time-user-error)))

(defun jirascope-project-edit-graphical ()
  "Edit a project in a buffer."
  (interactive)
  (if (jirascope-dyn-get-installed)
    (jirascope-dyn-project-edit-graphical-interactive)
    (user-error jirascope--first-time-user-error)))

(defun jirascope-project-edit-finish ()
  "Finish editing a project in a buffer and send it to the server."
  (interactive)
  (if (jirascope-dyn-get-installed)
    (jirascope-dyn-project-edit-graphical-finish)
    (user-error jirascope--first-time-user-error)))

(defun jirascope-project-delete ()
  "Delete a project."
  (interactive)
  (if (jirascope-dyn-get-installed)
    (jirascope-dyn-project-delete-interactive)
    (user-error jirascope--first-time-user-error)))

(defun jirascope-tree-open()
  "Open the project tree buffer."
  (interactive)
  (if (jirascope-dyn-get-installed)
    (jirascope-dyn-state-open)
    (user-error jirascope--first-time-user-error)))

(defface jirascope-issue-key
  '((t (:inherit info-title-1)))
  "Face used for issue key headers."
  :group 'jirascope)

(defface jirascope-project-key
  '((t (:inherit info-title-1)))
  "Face used for project key headers."
  :group 'jirascope)

(defface jirascope-diff-alert
    '((t (:inherit font-lock-warning-face)))
    "Face used for issue diff alerts."
    :group 'jirascope)

(defface jirascope-diff-new
    '((t (:inherit font-lock-string-face)))
    "Face used for issue new diff."
    :group 'jirascope)

(defface jirascope-diff-old
    '((t (:inherit font-lock-doc-markup-face)))
    "Face used for issue old diff."
    :group 'jirascope)

(define-button-type 'jirascope-issue-button
    'follow-link t
    'action 'jirascope-dyn-issue-button-action)

(define-button-type 'jirascope-project-button
    'follow-link t
    'action 'jirascope-dyn-project-button-action)

(defun jirascope-insert-button (text ty)
  "Used by jirascope-dyn.
Create a button with the given TEXT and type TY in the current buffer."
  (insert-button text
    :type ty))


(provide 'jirascope)

;;; jirascope.el ends here
