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
(defun jiroscope-delete-issue ()
  (interactive)
  (jiroscope-dyn-delete-issue-interactive))

(defun jiroscope-display-issue ()
  (interactive)
  (jiroscope-dyn-display-issue-interactive))

(defun jiroscope-create-issue ()
  (interactive)
  (jiroscope-dyn-create-issue))

(defun jiroscope-edit-issue ()
  (interactive)
  (jiroscope-dyn-edit-issue))

(defun jiroscope-transition-issue ()
  (interactive)
  (jiroscope-dyn-transition-issue-interactive))

(defface jiroscope-issue-key
  '((t (:inherit info-title-1)))
  "Face used for issue key headers."
  :group 'jiroscope)

(provide 'jiroscope)
