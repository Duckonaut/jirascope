;;; jirascope-dyn-get.el --- Utilities to obtain jirascope-dyn -*- lexical-binding: t; coding: utf-8 -*-

;; SPDX-License-Identifier: MIT OR Apache-2.0

;;; Commentary:
;; Based heavily on:
;; https://github.com/emacs-tree-sitter/elisp-tree-sitter/blob/master/core/tsc-dyn-get.el

;; This file contains the utilities to obtain the dynamic module `jirascope-dyn', by
;; either downloading pre-built binaries or building from source.

;;; Code:

(require 'seq)
(require 'dired-aux)
(require 'compile)

(eval-when-compile
  (require 'subr-x)
  (require 'cl-lib))

(eval-when-compile
  ;; Version string set by `jirascope-dyn' when it's loaded.
  (defvar jirascope-dyn--version)
  ;; Directory where `jirascope-dyn' is located (custom, defined in `jirascope.el').
  (defvar jirascope-dyn-dir))


(defun jirascope-dyn-get--internet-connectivity ()
  "Return t if we have internet connectivity, nil otherwise."
  (condition-case _
      (url-retrieve-synchronously "https://github.com")
    (error nil)))

(defun jirascope-dyn-get--can-fetch (version)
  "Return t if we can fetch the pre-built binary for VERSION, nil otherwise."
  ;; perform a HEAD request to check if the file exists.
  (let ((url-request-method "HEAD")
        (url (format
               "https://github.com/Duckonaut/jirascope/releases/download/v%s/jirascope-dyn.so"
               version)))
    (ignore url-request-method)
    (condition-case _
        (with-current-buffer (url-retrieve-synchronously url)
          (goto-char (point-min))
          ;; we're happy with a 200 or 302 response.
          (or (looking-at "HTTP/.* 200")
              (looking-at "HTTP/.* 302")))
        (error nil))))

(defun jirascope-dyn-get--can-build ()
  "Return t if we can build the dynamic module, nil otherwise."
  (executable-find "cargo"))

(defun jirascope-dyn-get-available ()
  "Returns t if we either a valid `jirascope-dyn' binary or can build/fetch one."
  ;; Check if we have a valid binary.
  (or (jirascope-dyn-get--try-load)
      ;; check if we have internet connectivity.
      (and (jirascope-dyn-get--internet-connectivity)
           ;; check if we can build/fetch a binary.
           (or (jirascope-dyn-get--can-fetch (jirascope-dyn-get--recorded-version))
               (jirascope-dyn-get--can-build)))))

(defun jirascope-dyn-get-installed ()
  "Return t if we have a valid `jirascope-dyn' binary, nil otherwise."
  (jirascope-dyn-get--try-load))

(defun jirascope-dyn-get-loaded ()
  "Return t if `jirascope-dyn' is loaded, nil otherwise."
  (featurep 'jirascope-dyn))

(defun jirascope-dyn-get-install (version)
  "Install the dynamic module `jirascope-dyn' with VERSION."
  (if (jirascope-dyn-get-available)
    (jirascope-dyn-get-ensure version)
    (error "No viable installation method found")))

(defconst jirascope-dyn-get--version-file "DYN-VERSION"
  "File that records the version after getting the binary from a source.")

(defcustom jirascope-dyn-get-from '(:github :compilation)
  "Where the dynamic module binary should come from, in order of priority.

For pre-built binaries, it attempts to download the requested version.

For local compilation, the Rust toolchain is required.

If you want to manually get the dynamic module through another mechanism,
instead of letting `jirascope-dyn-get' automatically try to download/build it,
set this to nil."
  :group 'jirascope
  :type '(set (const :tag "Binary from GitHub" :github)
              (const :tag "Local Compilation" :compilation)))

(defvar jirascope-dyn-get--force-sync nil)

(defun jirascope-dyn-get--dir ()
  "Return the directory to put `jirascope-dyn' module in."
  (or jirascope-dyn-dir
      (error "Could not locate the directory for `jirascope-dyn'")))

(defun jirascope-dyn-get--ext ()
  "Return the dynamic module extension, which is system-dependent."
  (pcase system-type
    ('windows-nt "dll")
    ('darwin "dylib")
    ((or 'gnu 'gnu/linux 'gnu/kfreebsd) "so")
    ((or 'ms-dos 'cygwin) (error "Unsupported system-type %s" system-type))
    (_ "so")))

(defun jirascope-dyn-get--file ()
  "Return the dynamic module filename, which is OS-dependent."
  (format "jirascope-dyn.%s" (jirascope-dyn-get--ext)))

;;; TODO: Make this correct.
(defun jirascope-dyn-get--system-specific-file ()
  "Return the dynamic module filename, which is system-dependent."
  (pcase system-type
    ('windows-nt "jirascope-dyn.x86_64-pc-windows-msvc.dll")
    ('darwin (if (string-prefix-p "x86_64" system-configuration)
                 "jirascope-dyn.x86_64-apple-darwin.dylib"
               "jirascope-dyn.aarch64-apple-darwin.dylib"))
    ((or 'gnu 'gnu/linux 'gnu/kfreebsd)
     "jirascope-dyn.x86_64-unknown-linux-gnu.so")))

(defun jirascope-dyn-get--log (format-string &rest args)
  "Log a message to the `jirascope-dyn-get' logger.
FORMAT-STRING and ARGS are passed to `message'."
  (apply #'message (concat "jirascope-dyn-get: " format-string) args))

(defun jirascope-dyn-get--warn (&rest args)
  "Log a warning to the `jirascope-dyn-get' logger.
ARGS are passed to `format'."
  (display-warning 'jirascope-dyn-get (apply #'format args) :emergency))

(defun jirascope-dyn-get--recorded-version ()
  "Return the `jirascope-dyn' version recorded.
The information is stored in the file `jirascope-dyn-get--version-file'."
  (let ((default-directory (jirascope-dyn-get--dir)))
    (when (file-exists-p jirascope-dyn-get--version-file)
      (with-temp-buffer
        (let ((coding-system-for-read 'utf-8))
          (insert-file-contents jirascope-dyn-get--version-file)
          (buffer-string))))))

(defun jirascope-dyn-get--loaded-version ()
  "Return the currently loaded version of `jirascope-dyn'."
  (and (featurep 'jirascope-dyn) (bound-and-true-p jirascope-dyn--version)))

;; ----------------------------------------------------------------------------
;; Pre-built binaries downloaded through HTTP.

(defun jirascope-dyn-get--check-http (&rest _args)
  "Check the HTTP status code of the current `url-copy-file' request."
  (when-let ((status (bound-and-true-p url-http-response-status)))
    (when (>= status 400)
      (error "Got HTTP status code %s" status))))

;; TODO: Find a better way to make `url-copy-file' handle bad HTTP status codes.
(defun jirascope-dyn-get--url-copy-file (&rest args)
  "A wrapper around `url-copy-file' that signals errors for bad HTTP statuses.
ARGS are passed to `url-copy-file'."
  (advice-add 'mm-dissect-buffer :before #'jirascope-dyn-get--check-http)
  (unwind-protect
      (apply #'url-copy-file args)
    (advice-remove 'mm-dissect-buffer #'jirascope-dyn-get--check-http)))

(defun jirascope-dyn-get--github (version)
  "Download the pre-compiled VERSION of `jirascope-dyn' from GitHub.
This function records the downloaded version in the manifest
`jirascope-dyn-get--version-file'."
  (let* ((bin-dir (jirascope-dyn-get--dir))
         (default-directory bin-dir)
         (_ (unless (file-directory-p bin-dir) (make-directory bin-dir)))
         (local-name (jirascope-dyn-get--file))
         (remote-name local-name)
         (url-request-method "GET")
         (url (format
                "https://github.com/Duckonaut/jirascope/releases/download/v%s/%s"
                version
                remote-name)))
    (ignore url-request-method)
    (jirascope-dyn-get--log "Downloading %s" url)
    (jirascope-dyn-get--url-copy-file url local-name :ok-if-already-exists)
    (with-temp-file jirascope-dyn-get--version-file
      (let ((coding-system-for-write 'utf-8))
        (insert version)))))

;; ----------------------------------------------------------------------------
;; Local compilation.

(define-error 'jirascope-dyn-get--compile-error "Could not compile `jirascope-dyn'")

(defun jirascope-dyn-get--build-output (face &rest args)
  "Print a message to the `jirascope-dyn-get' logger.
Message is printed with FACE, which is passed to `propertize'.
ARGS are passed to `format'."
  (declare (indent 1))
  (let ((str (propertize (apply #'format args) 'face face 'font-lock-face face))
        (inhibit-read-only t))
    (if noninteractive
        (progn (princ str) (princ "\n"))
      (insert str)
      (insert "\n"))))

(defun jirascope-dyn-get--print-stdout (_proc string)
  "Print STRING to stdout."
  (princ string))

(defmacro jirascope-dyn-get--compilation-to-stdout (condition &rest body)
  "Eval BODY forms with compilation output conditionally redirected to `princ'.
If CONDITION is true, redirect compilation output to `princ' instead of the
compilation buffer. Otherwise, eval BODY forms normally."
  (declare (indent 1))
  `(if ,condition
    (advice-add 'compilation-filter :override #'jirascope-dyn-get--print-stdout)
    (unwind-protect
      (progn ,@body)
      (advice-remove 'compilation-filter #'jirascope-dyn-get--print-stdout))
    ,@body))

(defun jirascope-dyn-get--build-version ()
  "Return the dynamic module's version after asking `cargo'."
  (thread-first (shell-command-to-string "cargo pkgid")
    string-trim
    (split-string "\[#:\]")
    last car))

;; TODO: Remove this when cargo allows specifying output file name.
(defun jirascope-dyn-get--out-file ()
  "Return cargo's output filename, which is system-dependent."
  (let ((base (pcase system-type
                ('windows-nt "jirascope_dyn")
                (_ "libjirascope_dyn"))))
    (format "%s.%s" base (jirascope-dyn-get--ext))))

(defun jirascope-dyn-get--build-cleanup (comp-buffer status)
  "Clean up after compiling the dynamic module `jirascope-dyn'.
This function copies the built binary to the appropriate location, delete the
build directory, and record the built version in the manifest
`jirascope-dyn-get--version-file'.
COMP-BUFFER is the compilation buffer. STATUS is the exit status of the
compilation process."
  (with-current-buffer comp-buffer
    (let* ((file (jirascope-dyn-get--file))
           (out-name (jirascope-dyn-get--out-file))
           (out-file (format "target/release/%s" out-name)))
      (unless (string= status "finished\n")
        (signal 'jirascope-dyn-get--compile-error
                (list (format "Compilation failed with status: %s" status))))
      (jirascope-dyn-get--build-output 'compilation-info
        "Moving binary %s from build dir" out-name)
      (condition-case _
          (rename-file out-file file)
        (file-already-exists
         (delete-file file)
         (rename-file out-file file)))
      (jirascope-dyn-get--build-output 'compilation-info
        "Removing build dir")
      (delete-directory "target" :recursive)
      (jirascope-dyn-get--build-output 'compilation-info
        "Recording built version in %s" jirascope-dyn-get--version-file)
      (with-temp-file jirascope-dyn-get--version-file
        (let ((coding-system-for-write 'utf-8))
          (insert (jirascope-dyn-get--build-version))))
      (jirascope-dyn-get--build-output 'success "Done"))))

;; XXX: We don't use `call-process' because the process it creates is not killed
;; when Emacs exits in batch mode. That's probably an Emacs's bug.
(defun jirascope-dyn-get--build-sync (dir)
  "Build the dynamic module `jirascope-dyn' and put it in DIR, blocking until done."
  ;; FIX: Figure out how to print the progress bar when run synchronously.
  (jirascope-dyn-get--compilation-to-stdout noninteractive
    (let ((proc (jirascope-dyn-get--build-async dir)))
      (condition-case s
          (while (accept-process-output proc)
            (unless noninteractive
              (redisplay)))
        (quit (let ((buf (process-buffer proc)))
                (set-process-query-on-exit-flag proc nil)
                (interrupt-process proc)
                (with-current-buffer buf
                  (jirascope-dyn-get--build-output 'error "Cancelled")
                  ;; TODO: Don't wait for a fixed amount of time.
                  (sit-for 1)
                  (kill-buffer)))
              (signal (car s) (cdr s)))))))

(defun jirascope-dyn-get--build-async (dir)
  "Build the dynamic module `jirascope-dyn' and put it in DIR, asynchrounously."
  (let* ((default-directory dir)
         (compilation-auto-jump-to-first-error nil)
         (compilation-scroll-output t)
         ;; We want responsive progress bar. It's ok since the output is small.
         (process-adaptive-read-buffering nil)
         (comp-buffer (compilation-start
                       "cargo build --release"
                       nil (lambda (_) "*jirascope-dyn compilation*")))
         (proc (get-buffer-process comp-buffer)))
    (with-current-buffer comp-buffer
      (setq-local compilation-error-regexp-alist nil)
      (add-hook 'compilation-finish-functions #'jirascope-dyn-get--build-cleanup
                nil :local)
      (unless noninteractive
        (when (functionp 'ansi-color-apply-on-region)
          (add-hook 'compilation-filter-hook
            (lambda () (ansi-color-apply-on-region (point-min) (point-max)))
            nil :local))))
    proc))

(defun jirascope-dyn-get--build (&optional dir)
  "Build the dynamic module `jirascope-dyn' from source.

If DIR is nil, use `jirascope-dyn-get--dir'.
Otherwise, use DIR as the build directory.

When called during an attempt to load `jirascope', or in batch mode,
this blocks until compilation finishes. In other situations, it runs
in the background.

This function records the built version in the manifest
`jirascope-dyn-get--version-file'.

On Windows, if `jirascope-dyn' has already been loaded, compilation will fail
because the OS doesn't allow overwriting opened dynamically-loaded libraries."
  (unless dir (setq dir jirascope-dyn-dir))
  (while (not (executable-find "cargo"))
    (if noninteractive
        (signal 'jirascope-dyn-get--compile-error "Could not find `cargo' executable")
      ;; TODO: Make a better prompt.
      (unless
        (y-or-n-p
          "Could not find `cargo' executable.
Do you want to install the rust toolchain?")

        (signal 'jirascope-dyn-get--compile-error "Compilation was cancelled"))))
  (if (or noninteractive
          (not (featurep 'jirascope-dyn))
          jirascope-dyn-get--force-sync)
      (jirascope-dyn-get--build-sync dir)
    ;; TODO: Notify user for further actions. If `jirascope' has not been loaded,
    ;; offer to load it. If it has already been loaded, offer to restart Emacs
    ;; to be able to load the newly built `jirascope-dyn'.
    (jirascope-dyn-get--build-async dir)))

;; ----------------------------------------------------------------------------
;; Generic mechanism.

(defun jirascope-dyn-get--module-load-no-error (file)
  "Try loading `jirascope-dyn' from FILE.
Return nil if the file does not exist, or is not a loadable shared library."
  (or (featurep 'jirascope-dyn)
      (condition-case _
          (module-load file)
        (module-open-failed nil))))

;; On macOS, we use`.dylib', which is more sensible than `.so'.
;;
;; XXX: Using `require' after setting`module-file-suffix' to `.dylib' results in
;; "Cannot open load file: No such file or directory, jirascope-dyn".
;;
;; XXX: Using `load' results in an error message with garbled text: "Symbol’s
;; value as variable is void: Ïúíþ".
;;
;; Therefore, we need to search for the file and use `module-load' directly.
(defun jirascope-dyn-get--try-load-mac ()
  "Search and load the dynamic module on macOS."
  (let ((file "jirascope-dyn.dylib"))
    ;; Try directory containing `load-file-name'. Typical case. TODO: Remove
    ;; this special case.
    (when load-file-name
      (jirascope-dyn-get--module-load-no-error (concat (file-name-directory load-file-name)
                                        file)))
    ;; Try working directory (e.g. when invoked by `cask'). TODO: Modifying load
    ;; path when using `cask' instead.
    (jirascope-dyn-get--module-load-no-error file)
    ;; Fall back to `load-path'.
    (seq-find (lambda (dir)
                (let ((full-name (concat (file-name-as-directory
                                          (expand-file-name dir))
                                         file)))
                  (jirascope-dyn-get--module-load-no-error full-name)))
              load-path)))

(defun jirascope-dyn-get--try-load ()
  "Try loading `jirascope-dyn' without signaling an error.
Return t on success, nil otherwise."
  (if (featurep 'jirascope-dyn)
      t
    (when (eq system-type 'darwin)
      (jirascope-dyn-get--try-load-mac))
    (require 'jirascope-dyn nil :noerror)))

;; TODO: Add tests for this.
(defun jirascope-dyn-get-ensure (requested)
  "Try to get and load the REQUESTED (or later) version of `jirascope-dyn'.

If this function cannot find a suitable version on `load-path', it tries to get
the dynamic module from sources listed in `jirascope-dyn-get-from'.

NOTE: Emacs cannot unload dynamic modules, so if `jirascope-dyn' was already
loaded, you will need to restart Emacs to load the new version."
  (let* ((default-directory (jirascope-dyn-get--dir))
         (recorded (jirascope-dyn-get--recorded-version))
         (loaded (jirascope-dyn-get--loaded-version))
         (load-path (cons (jirascope-dyn-get--dir) load-path))
         (jirascope-dyn-get--force-sync t)
         get-new)
    (cl-block nil
      (dolist (source jirascope-dyn-get-from)
        (jirascope-dyn-get--log "Using source %s (:loaded %s :recorded %s :requested %s)"
                          source loaded recorded requested)
        (setq get-new (pcase source
                        (:github (lambda () (jirascope-dyn-get--github requested)))
                        (:compilation (lambda () (jirascope-dyn-get--build)))
                        (_ (error "Don't know how to get `jirascope-dyn' from source %s" source))))
        (with-demoted-errors "Could not get `jirascope-dyn': %s"
          (cond
           (loaded (if (version<= requested loaded)
                       (jirascope-dyn-get--log "Loaded version already satisfies requested -> skipping")
                     ;; TODO: On Windows, refuse to continue and ask user to set
                     ;; the requested version and restart instead.
                     (jirascope-dyn-get--log "Loaded version is older than requested -> getting new")
                     (funcall get-new)))
           (recorded (if (version<= requested recorded)
                         (progn
                           (jirascope-dyn-get--log "Recorded version already satifies requested -> loading")
                           (unless (jirascope-dyn-get--try-load)
                             ;; The version file may have been accidentally deleted.
                             (jirascope-dyn-get--log "Could not load -> getting new")
                             (funcall get-new)
                             (jirascope-dyn-get--try-load)))
                       (jirascope-dyn-get--log "Recorded version is older than requested -> getting new")
                       (funcall get-new)
                       (jirascope-dyn-get--try-load)))
           (t (funcall get-new)
              (jirascope-dyn-get--try-load)))
          (when (featurep 'jirascope-dyn)
            (cl-return)))))
    (if (and loaded (version< loaded requested))
        (jirascope-dyn-get--warn "Version %s is requested, but %s was already loaded. Please try restarting Emacs."
                           requested loaded)
      ;; Even if none of the sources worked, the module may still be there.
      (jirascope-dyn-get--try-load)
      (if-let ((loaded (jirascope-dyn-get--loaded-version)))
          (when (version< loaded requested)
            (jirascope-dyn-get--warn "Version %s is requested, but actual version after loading is %s."
                               requested loaded))
        (jirascope-dyn-get--warn "Failed to get requested version %s." requested)))
    (jirascope-dyn-get--loaded-version)))

(provide 'jirascope-dyn-get)
;;; jirascope-dyn-get.el ends here
