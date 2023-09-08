;;; jiroscope-dyn-get.el --- Utilities to obtain jiroscope-dyn -*- lexical-binding: t; coding: utf-8 -*-

;;; Based heavily on:
;;; https://github.com/emacs-tree-sitter/elisp-tree-sitter/blob/master/core/tsc-dyn-get.el

;;; Commentary:

;; This file contains the utilities to obtain the dynamic module `jiroscope-dyn', by
;; either downloading pre-built binaries or building from source.

;;; Code:

(require 'seq)
(require 'dired-aux)
(require 'compile)

(eval-when-compile
  (require 'subr-x)
  (require 'cl-lib))

(eval-when-compile
  ;; Version string set by `jiroscope-dyn' when it's loaded.
  (defvar jiroscope-dyn--version))

(defconst jiroscope-dyn-get--version-file "DYN-VERSION"
  "File that records the version after getting the binary from a source.")

(defconst jiroscope--dir (file-name-directory (or (locate-library "jiroscope.el") ""))
  "The directory where the library `jiroscope' is located.")

(defgroup jiroscope nil
  "Core jiroscope APIs.")

(defcustom jiroscope-dyn-dir jiroscope--dir
  "The directory that `jiroscope-dyn' module is resided.
This should be set before `jiroscope' is loaded.

Example setting:
\(setq jiroscope-dyn-dir (expand-file-name \"jiroscope/\" user-emacs-directory))"
  :group 'jiroscope
  :type 'directory)

(defcustom jiroscope-dyn-get-from '(:github :compilation)
  "Where the dynamic module binary should come from, in order of priority.

For pre-built binaries, it attempts to download the requested version.

For local compilation, the Rust toolchain is required.

If you want to manually get the dynamic module through another mechanism,
instead of letting `jiroscope-dyn-get' automatically try to download/build it, set
this to nil."
  :group 'jiroscope
  :type '(set (const :tag "Binary from GitHub" :github)
              (const :tag "Local Compilation" :compilation)))

(defvar jiroscope-dyn-get--force-sync nil)

(defun jiroscope-dyn-get--dir ()
  "Return the directory to put `jiroscope-dyn' module in."
  (or jiroscope-dyn-dir
      (error "Could not locate the directory for `jiroscope-dyn'")))

(defun jiroscope-dyn-get--ext ()
  "Return the dynamic module extension, which is system-dependent."
  (pcase system-type
    ('windows-nt "dll")
    ('darwin "dylib")
    ((or 'gnu 'gnu/linux 'gnu/kfreebsd) "so")
    ((or 'ms-dos 'cygwin) (error "Unsupported system-type %s" system-type))
    (_ "so")))

(defun jiroscope-dyn-get--file ()
  "Return the dynamic module filename, which is OS-dependent."
  (format "jiroscope-dyn.%s" (jiroscope-dyn-get--ext)))

;;; TODO: Make this correct.
(defun jiroscope-dyn-get--system-specific-file ()
  "Return the dynamic module filename, which is system-dependent."
  (pcase system-type
    ('windows-nt "jiroscope-dyn.x86_64-pc-windows-msvc.dll")
    ('darwin (if (string-prefix-p "x86_64" system-configuration)
                 "jiroscope-dyn.x86_64-apple-darwin.dylib"
               "jiroscope-dyn.aarch64-apple-darwin.dylib"))
    ((or 'gnu 'gnu/linux 'gnu/kfreebsd)
     "jiroscope-dyn.x86_64-unknown-linux-gnu.so")))

(defun jiroscope-dyn-get--log (format-string &rest args)
  (apply #'message (concat "jiroscope-dyn-get: " format-string) args))

(defun jiroscope-dyn-get--warn (&rest args)
  (display-warning 'jiroscope-dyn-get (apply #'format args) :emergency))

(defun jiroscope-dyn-get--recorded-version ()
  "Return the `jiroscope-dyn' version recorded in the manifest
`jiroscope-dyn-get--version-file'."
  (let ((default-directory (jiroscope-dyn-get--dir)))
    (when (file-exists-p jiroscope-dyn-get--version-file)
      (with-temp-buffer
        (let ((coding-system-for-read 'utf-8))
          (insert-file-contents jiroscope-dyn-get--version-file)
          (buffer-string))))))

(defun jiroscope-dyn-get--loaded-version ()
  "Return the currently loaded version of `jiroscope-dyn'."
  (and (featurep 'jiroscope-dyn) (bound-and-true-p jiroscope-dyn--version)))

;;; ----------------------------------------------------------------------------
;;; Pre-built binaries downloaded through HTTP.

(defun jiroscope-dyn-get--check-http (&rest _args)
  (when-let ((status (bound-and-true-p url-http-response-status)))
    (when (>= status 400)
      (error "Got HTTP status code %s" status))))

;; TODO: Find a better way to make `url-copy-file' handle bad HTTP status codes.
(defun jiroscope-dyn-get--url-copy-file (&rest args)
  "A wrapper around `url-copy-file' that signals errors for bad HTTP statuses."
  (advice-add 'mm-dissect-buffer :before #'jiroscope-dyn-get--check-http)
  (unwind-protect
      (apply #'url-copy-file args)
    (advice-remove 'mm-dissect-buffer #'jiroscope-dyn-get--check-http)))

(defun jiroscope-dyn-get--github (version)
  "Download the pre-compiled VERSION of `jiroscope-dyn' from GitHub.
This function records the downloaded version in the manifest
`jiroscope-dyn-get--version-file'."
  (let* ((bin-dir (jiroscope-dyn-get--dir))
         (default-directory bin-dir)
         (_ (unless (file-directory-p bin-dir) (make-directory bin-dir)))
         (local-name (jiroscope-dyn-get--file))
         (remote-name local-name)
         (url (format "https://github.com/duckonaut/jiroscope/releases/download/%s/%s"
                      version remote-name)))
    (jiroscope-dyn-get--log "Downloading %s" url)
    (jiroscope-dyn-get--url-copy-file url local-name :ok-if-already-exists)
    (with-temp-file jiroscope-dyn-get--version-file
      (let ((coding-system-for-write 'utf-8))
        (insert version)))))

;;; ----------------------------------------------------------------------------
;;; Local compilation.

(define-error 'jiroscope-compile-error "Could not compile `jiroscope-dyn'")

(defun jiroscope-dyn-get--build-output (face &rest args)
  (declare (indent 1))
  (let ((str (propertize (apply #'format args) 'face face 'font-lock-face face))
        (inhibit-read-only t))
    (if noninteractive
        (progn (princ str) (princ "\n"))
      (insert str)
      (insert "\n"))))

(defmacro jiroscope-dyn-get--compilation-to-stdout (condition &rest body)
  "Eval BODY forms with compilation output conditionally redirected to `princ'."
  (declare (indent 1))
  (let ((print-stdout (make-symbol "print-stdout")))
    `(if ,condition
         (let ((,print-stdout (lambda (_proc string) (princ string))))
           (advice-add 'compilation-filter :override ,print-stdout)
           (unwind-protect
               (progn ,@body)
             (advice-remove 'compilation-filter ,print-stdout)))
       ,@body)))

(defun jiroscope-dyn-get--build-version ()
  "Return the dynamic module's version after asking 'cargo'."
  (thread-first (shell-command-to-string "cargo pkgid")
    string-trim
    (split-string "\[#:\]")
    last car))

;; TODO: Remove this when cargo allows specifying output file name.
(defun jiroscope-dyn-get--out-file ()
  "Return cargo's output filename, which is system-dependent."
  (let ((base (pcase system-type
                ('windows-nt "jiroscope_dyn")
                (_ "libjiroscope_dyn"))))
    (format "%s.%s" base (jiroscope-dyn-get--ext))))

(defun jiroscope-dyn-get--build-cleanup (comp-buffer status)
  "Clean up after compiling the dynamic module `jiroscope-dyn'.
This function copies the built binary to the appropriate location, delete the
build directory, and record the built version in the manifest
`jiroscope-dyn-get--version-file'."
  (with-current-buffer comp-buffer
    (let* ((file (jiroscope-dyn-get--file))
           (out-name (jiroscope-dyn-get--out-file))
           (out-file (format "target/release/%s" out-name)))
      (unless (string= status "finished\n")
        (signal 'jiroscope-compile-error
                (list (format "Compilation failed with status: %s" status))))
      (jiroscope-dyn-get--build-output 'compilation-info
        "Moving binary %s from build dir" out-name)
      (condition-case _
          (rename-file out-file file)
        (file-already-exists
         (delete-file file)
         (rename-file out-file file)))
      (jiroscope-dyn-get--build-output 'compilation-info
        "Removing build dir")
      (delete-directory "target" :recursive)
      (jiroscope-dyn-get--build-output 'compilation-info
        "Recording built version in %s" jiroscope-dyn-get--version-file)
      (with-temp-file jiroscope-dyn-get--version-file
        (let ((coding-system-for-write 'utf-8))
          (insert (jiroscope-dyn-get--build-version))))
      (jiroscope-dyn-get--build-output 'success "Done"))))

;; XXX: We don't use `call-process' because the process it creates is not killed
;; when Emacs exits in batch mode. That's probably an Emacs's bug.
(defun jiroscope-dyn-get--build-sync (dir)
  "Build the dynamic module `jiroscope-dyn' and put it in DIR, blocking until done."
  ;; FIX: Figure out how to print the progress bar when run synchronously.
  (jiroscope-dyn-get--compilation-to-stdout noninteractive
    (let ((proc (jiroscope-dyn-get--build-async dir)))
      (condition-case s
          (while (accept-process-output proc)
            (unless noninteractive
              (redisplay)))
        (quit (let ((buf (process-buffer proc)))
                (set-process-query-on-exit-flag proc nil)
                (interrupt-process proc)
                (with-current-buffer buf
                  (jiroscope-dyn-get--build-output 'error "Cancelled")
                  ;; TODO: Don't wait for a fixed amount of time.
                  (sit-for 1)
                  (kill-buffer)))
              (signal (car s) (cdr s)))))))

(defun jiroscope-dyn-get--build-async (dir)
  "Build the dynamic module `jiroscope-dyn' and put it in DIR, asynchrounously."
  (let* ((default-directory dir)
         (compilation-auto-jump-to-first-error nil)
         (compilation-scroll-output t)
         ;; We want responsive progress bar. It's ok since the output is small.
         (process-adaptive-read-buffering nil)
         (comp-buffer (compilation-start
                       "cargo build --release"
                       nil (lambda (_) "*jiroscope-dyn compilation*")))
         (proc (get-buffer-process comp-buffer)))
    (with-current-buffer comp-buffer
      (setq-local compilation-error-regexp-alist nil)
      (add-hook 'compilation-finish-functions #'jiroscope-dyn-get--build-cleanup
                nil :local)
      (unless noninteractive
        (when (functionp 'ansi-color-apply-on-region)
          (add-hook 'compilation-filter-hook
            (lambda () (ansi-color-apply-on-region (point-min) (point-max)))
            nil :local))))
    proc))

(defun jiroscope-dyn-get--build (&optional dir)
  "Build the dynamic module `jiroscope-dyn' from source.

When called during an attempt to load `jiroscope', or in batch mode, this blocks until
compilation finishes. In other situations, it runs in the background.

This function records the built version in the manifest
`jiroscope-dyn-get--version-file'.

On Windows, if `jiroscope-dyn' has already been loaded, compilation will fail because
the OS doesn't allow overwriting opened dynamically-loaded libraries."
  (unless dir (setq dir jiroscope--dir))
  (while (not (executable-find "cargo"))
    (if noninteractive
        (signal 'jiroscope-compile-error "Could not find `cargo' executable")
      ;; TODO: Make a better prompt.
      (unless (y-or-n-p
               (format "Could not find `cargo' executable.
Please press '%s' after installing the Rust toolchain (e.g. from https://rustup.rs/).
Press '%s' to cancel. "
                       (propertize "y" 'face 'bold)
                       (propertize "n" 'face 'error)))
        (signal 'jiroscope-compile-error "Compilation was cancelled"))))
  (if (or noninteractive
          (not (featurep 'jiroscope-dyn))
          jiroscope-dyn-get--force-sync)
      (jiroscope-dyn-get--build-sync dir)
    ;; TODO: Notify user for further actions. If `jiroscope' has not been loaded,
    ;; offer to load it. If it has already been loaded, offer to restart Emacs
    ;; to be able to load the newly built `jiroscope-dyn'.
    (jiroscope-dyn-get--build-async dir)))

;;; ----------------------------------------------------------------------------
;;; Generic mechanism.

(defun jiroscope--module-load-noerror (file)
  "Try loading `jiroscope-dyn' from FILE.
Return nil if the file does not exist, or is not a loadable shared library."
  (or (featurep 'jiroscope-dyn)
      (condition-case _
          (module-load file)
        (module-open-failed nil))))

;; On macOS, we use`.dylib', which is more sensible than `.so'.
;;
;; XXX: Using `require' after setting`module-file-suffix' to `.dylib' results in
;; "Cannot open load file: No such file or directory, jiroscope-dyn".
;;
;; XXX: Using `load' results in an error message with garbled text: "Symbol’s
;; value as variable is void: Ïúíþ".
;;
;; Therefore, we need to search for the file and use `module-load' directly.
(defun jiroscope-dyn--try-load-mac ()
  "Search and load the dynamic module on macOS."
  (let ((file "jiroscope-dyn.dylib"))
    ;; Try directory containing `load-file-name'. Typical case. TODO: Remove
    ;; this special case.
    (when load-file-name
      (jiroscope--module-load-noerror (concat (file-name-directory load-file-name)
                                        file)))
    ;; Try working directory (e.g. when invoked by `cask'). TODO: Modifying load
    ;; path when using `cask' instead.
    (jiroscope--module-load-noerror file)
    ;; Fall back to `load-path'.
    (seq-find (lambda (dir)
                (let ((full-name (concat (file-name-as-directory
                                          (expand-file-name dir))
                                         file)))
                  (jiroscope--module-load-noerror full-name)))
              load-path)))

(defun jiroscope-dyn--try-load ()
  "Try loading `jiroscope-dyn' without signaling an error.
Return t on success, nil otherwise."
  (if (featurep 'jiroscope-dyn)
      t
    (when (eq system-type 'darwin)
      (jiroscope-dyn--try-load-mac))
    (require 'jiroscope-dyn nil :noerror)))

;; TODO: Add tests for this.
(defun jiroscope-dyn-get-ensure (requested)
  "Try to get and load the REQUESTED (or later) version of `jiroscope-dyn'.

If this function cannot find a suitable version on `load-path', it tries to get
the dynamic module from sources listed in `jiroscope-dyn-get-from'.

NOTE: Emacs cannot unload dynamic modules, so if `jiroscope-dyn' was already loaded,
you will need to restart Emacs to load the new version."
  (let* ((default-directory (jiroscope-dyn-get--dir))
         (recorded (jiroscope-dyn-get--recorded-version))
         (loaded (jiroscope-dyn-get--loaded-version))
         (load-path (cons (jiroscope-dyn-get--dir) load-path))
         (jiroscope-dyn-get--force-sync t)
         get-new)
    (cl-block nil
      (dolist (source jiroscope-dyn-get-from)
        (jiroscope-dyn-get--log "Using source %s (:loaded %s :recorded %s :requested %s)"
                          source loaded recorded requested)
        (setq get-new (pcase source
                        (:github (lambda () (jiroscope-dyn-get--github requested)))
                        (:compilation (lambda () (jiroscope-dyn-get--build)))
                        (_ (error "Don't know how to get `jiroscope-dyn' from source %s" source))))
        (with-demoted-errors "Could not get `jiroscope-dyn': %s"
          (cond
           (loaded (if (version<= requested loaded)
                       (jiroscope-dyn-get--log "Loaded version already satisfies requested -> skipping")
                     ;; TODO: On Windows, refuse to continue and ask user to set
                     ;; the requested version and restart instead.
                     (jiroscope-dyn-get--log "Loaded version is older than requested -> getting new")
                     (funcall get-new)))
           (recorded (if (version<= requested recorded)
                         (progn
                           (jiroscope-dyn-get--log "Recorded version already satifies requested -> loading")
                           (unless (jiroscope-dyn--try-load)
                             ;; The version file may have been accidentally deleted.
                             (jiroscope-dyn-get--log "Could not load -> getting new")
                             (funcall get-new)
                             (jiroscope-dyn--try-load)))
                       (jiroscope-dyn-get--log "Recorded version is older than requested -> getting new")
                       (funcall get-new)
                       (jiroscope-dyn--try-load)))
           (t (funcall get-new)
              (jiroscope-dyn--try-load)))
          (when (featurep 'jiroscope-dyn)
            (cl-return)))))
    (if (and loaded (version< loaded requested))
        (jiroscope-dyn-get--warn "Version %s is requested, but %s was already loaded. Please try restarting Emacs."
                           requested loaded)
      ;; Even if none of the sources worked, the module may still be there.
      (jiroscope-dyn--try-load)
      (if-let ((loaded (jiroscope-dyn-get--loaded-version)))
          (when (version< loaded requested)
            (jiroscope-dyn-get--warn "Version %s is requested, but actual version after loading is %s."
                               requested loaded))
        (jiroscope-dyn-get--warn "Failed to get requested version %s." requested)))
    (jiroscope-dyn-get--loaded-version)))

(provide 'jiroscope-dyn-get)
;;; jiroscope-dyn-get.el ends here
