;;; jiroscope.el --- Core Jiroscope APIs -*- lexical-binding: t; coding: utf-8 -*-

(unless (functionp 'module-load)
  (error "Dynamic module feature not available, please compile Emacs --with-modules option turned on"))

;; Load the dynamic module at compile time as well, to satisfy the byte compiler.
(eval-and-compile
  (defconst jiroscope--dyn-version "0.1.0"
    "Required version of the dynamic module `jiroscope-dyn'.")
  (require 'jiroscope-dyn-get)
  (jiroscope-dyn-get-ensure jiroscope--dyn-version))

(defun jiroscope-i-am-here () (message "hi"))

(require 'jiroscope-dyn)

(provide 'jiroscope)
