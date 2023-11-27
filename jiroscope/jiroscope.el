;;; jiroscope.el --- Core Jiroscope APIs -*- lexical-binding: t; coding: utf-8 -*-

(unless (functionp 'module-load)
  (error "Dynamic module feature not available, please compile Emacs --with-modules option turned on"))

;; Load the dynamic module at compile time as well, to satisfy the byte compiler.
(eval-and-compile
  (defconst jiroscope--dyn-version "0.0.1"
    "Required version of the dynamic module `jiroscope-dyn'.")
  (require 'jiroscope-dyn-get)
  (jiroscope-dyn-get-ensure jiroscope--dyn-version))

(require 'jiroscope-dyn)

(defun jiroscope-setup (url login api_token)
  (jiroscope-dyn-setup url login api_token))

;; add bindings for interactive use
(defun jiroscope-issue-create ()
  (interactive)
  (jiroscope-dyn-issue-create-interactive))

(defun jiroscope-issue-display ()
  (interactive)
  (jiroscope-dyn-issue-display-interactive))

(defun jiroscope-issue-edit ()
  (interactive)
  (jiroscope-dyn-issue-edit-interactive))

(defun jiroscope-issue-edit-graphical ()
  (interactive)
  (jiroscope-dyn-issue-edit-graphical-interactive))

(defun jiroscope-issue-edit-finish ()
  (interactive)
  (jiroscope-dyn-issue-edit-graphical-finish))

(defun jiroscope-issue-delete ()
  (interactive)
  (jiroscope-dyn-issue-delete-interactive))

(defun jiroscope-issue-transition ()
  (interactive)
  (jiroscope-dyn-issue-transition-interactive))

(defun jiroscope-project-create ()
  (interactive)
  (jiroscope-dyn-project-create-interactive))

(defun jiroscope-project-edit ()
  (interactive)
  (jiroscope-dyn-project-edit-interactive))

(defun jiroscope-project-delete ()
  (interactive)
  (jiroscope-dyn-project-delete-interactive))

(defun jiroscope-tree-open()
  (interactive)
  (jiroscope-dyn-state-open))

(defface jiroscope-issue-key
  '((t (:inherit info-title-1)))
  "Face used for issue key headers."
  :group 'jiroscope)

(defface jiroscope-diff-alert
    '((t (:inherit font-lock-warning-face)))
    "Face used for issue diff alerts."
    :group 'jiroscope)

(defface jiroscope-diff-new
    '((t (:inherit font-lock-string-face)))
    "Face used for issue new diff."
    :group 'jiroscope)

(defface jiroscope-diff-old
    '((t (:inherit font-lock-doc-markup-face)))
    "Face used for issue old diff."
    :group 'jiroscope)

(define-button-type 'jiroscope-issue-button
    'follow-link t
    'action 'jiroscope-dyn-issue-button-action)

(defun jiroscope-insert-button (text ty)
  (insert-button text
    :type 'jiroscope-issue-button))

(provide 'jiroscope)
