;;; jiroscope.el --- Core Jiroscope APIs -*- lexical-binding: t; coding: utf-8 -*-

(unless (functionp 'module-load)
  (error "Dynamic module feature not available, please compile Emacs --with-modules option turned on"))

;; Load the dynamic module at compile time as well, to satisfy the byte compiler.
(eval-and-compile
  (defconst jiroscope--dyn-version "0.0.1"
    "Required version of the dynamic module `jiroscope-dyn'.")
  (require 'jiroscope-dyn-get)
  (jiroscope-dyn-get-ensure jiroscope--dyn-version))

;; take any number of arguments
(defun jiroscope-setup (url login api_token)
  (jiroscope-dyn-setup url login api_token))

(require 'jiroscope-dyn)

(provide 'jiroscope)
