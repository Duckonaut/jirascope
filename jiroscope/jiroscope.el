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

(defun jiroscope-issue-delete ()
  (interactive)
  (jiroscope-dyn-issue-delete-interactive))

(defun jiroscope-issue-transition ()
  (interactive)
  (jiroscope-dyn-issue-transition-interactive))

(defun jiroscope-project-create ()
  (interactive)
  (jiroscope-dyn-project-create-interactive))

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

(provide 'jiroscope)
