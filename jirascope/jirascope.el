;;; jirascope.el --- A Jira client -*- lexical-binding: t; coding: utf-8 -*-

;; SPDX-License-Identifier: MIT OR Apache-2.0
;; Author: Stanisław Zagórowski <duckonaut@gmail.com>
;; Version: 0.1.4
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

(defconst jirascope--dyn-version "0.1.4"
  "Required version of the dynamic module `jirascope-dyn'.")

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
  (progn (require 'jirascope-dyn)
    (defun jirascope-setup (url login api_token)
      "Setup Jirascope with the given cloud URL, LOGIN and API_TOKEN."
      (jirascope-dyn-setup url login api_token))

    ;; add bindings for interactive use
    (defun jirascope-issue-create ()
      "Create a new issue."
      (interactive)
      (jirascope-dyn-issue-create-interactive))

    (defun jirascope-issue-display ()
      "Display an issue."
      (interactive)
      (jirascope-dyn-issue-display-interactive))

    (defun jirascope-issue-edit ()
      "Edit an issue via prompt."
      (interactive)
      (jirascope-dyn-issue-edit-interactive))

    (defun jirascope-issue-edit-graphical ()
      "Edit an issue in a buffer."
      (interactive)
      (jirascope-dyn-issue-edit-graphical-interactive))

    (defun jirascope-issue-edit-finish ()
      "Finish editing an issue in a buffer and send it to the server."
      (interactive)
      (jirascope-dyn-issue-edit-graphical-finish))

    (defun jirascope-issue-delete ()
      "Delete an issue."
      (interactive)
      (jirascope-dyn-issue-delete-interactive))

    (defun jirascope-issue-transition ()
      "Transition an issue."
      (interactive)
      (jirascope-dyn-issue-transition-interactive))

    (defun jirascope-project-create ()
      "Create a project via prompt."
      (interactive)
      (jirascope-dyn-project-create-interactive))

    (defun jirascope-project-edit ()
      "Edit a project via prompt."
      (interactive)
      (jirascope-dyn-project-edit-interactive))

    (defun jirascope-project-edit-graphical ()
      "Edit a project in a buffer."
      (interactive)
      (jirascope-dyn-project-edit-graphical-interactive))

    (defun jirascope-project-edit-finish ()
      "Finish editing a project in a buffer and send it to the server."
      (interactive)
      (jirascope-dyn-project-edit-graphical-finish))

    (defun jirascope-project-delete ()
      "Delete a project."
      (interactive)
      (jirascope-dyn-project-delete-interactive))

    (defun jirascope-tree-open()
      "Open the project tree buffer."
      (interactive)
      (jirascope-dyn-state-open))

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
        :type ty)))
  (message "`jirascope-dyn' dynamic module not installed. If this is your first time using Jirascope, please run `M-x jirascope-install'"))


(provide 'jirascope)

;;; jirascope.el ends here
