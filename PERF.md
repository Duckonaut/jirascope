# Performance testing
When making an Emacs package, I wanted to make one using the Emacs module system, rather than
just in Emacs Lisp. This allows for using more powerful languages, like C++ or Rust
(or anything that can bind to C functions).

The Rust crate ecosystem seemed to have an interesting crate, called just `emacs` in the
[crates.io](https://crates.io) registry. It creates Rust <-> Emacs bindings in a simple way,
and since Rust statically links everything by default, this would let me use any other crates
I wanted while still creating a single binary without major tweaks to the default `cargo` build
system.

As the project requires internet connectivity, I had two choices. I could either use a package
already offerred in Emacs, like `url.el` or `request.el` which are very popular, or introduce a
Rust dependency. `request.el` has a pretty nice, relatively high level API. The most comparable
crate I found was [`ureq`](https://crates.io/crates/ureq). Both offer a high-level
"I Am Requesting This" kind of API, with blocking IO. The alternatives included
[`reqwest`](https://crates.io/crates/reqwest), which is another popular HTTP client library, but
`ureq` being designed around being blocking made the API simpler.

As Emacs is not designed to be multi-threaded, interacting with it through the module API
from multiple threads would have almost definitely caused race conditions in the underlying
Emacs code. The architecture of Emacs calling Rust also seemed like a giant hurdle to pulling
in a Rust async runtime like [`tokio`](https://crates.io/crates/tokio).

I may try to introduce *some* threading in the project to make everything more responsive later,
but I haven't experimented with that yet. Calling Emacs functions (required to do almost
anything useful) from another thread is not documented as "safe" anywhere, and I wouldn't assume
that it is. This is why a blocking request library like `ureq` seemed like a safe choice.

But, worried about the performance penalty of translating objects between Rust and Emacs Lisp
and the general delay due to the boundary of Emacs Lisp and Rust, I wanted to benchmark
`request.el` against `ureq`. There are four "tiers" of the integration I tested:

- **Exclusively Emacs Lisp**: The request is handled by `request.el`, and the processing and
  looping of it is done in Emacs Lisp.
- **Emacs Lisp using simple Rust functions**: The request and decoding of the HTTP request is
  done in Rust code, in simple single-request functions. The loop is done in Emacs Lisp.
- **Avoiding Emacs Lisp**: As much as possible is done from Rust, without crossing over the
  Emacs Lisp boundary.
- **Calling `request.el` from Rust**: This one is mainly a curiosity. It doesn't *really* make
  sense to use in practice for anything I can think of, but it should show the performance
  difference between the two libraries, ignoring the Emacs Lisp interpreter performance penalty
  where possible.

There are also two testing scenarios I ran through. A local test server (already on hand from
initial development of the package), and the remote Jira Cloud server. This would tell me
the "raw" performance of the approaches, and later show how much that actually matters in
a real world scenario. Both are simple GET requests, since the type of request and size of the
payload do not carry any real extra information to differentiate the approaches.

To be representative of the end goal, everything was called from Rust code, and the time
measurement done using the Rust standard `std::time` types.

An example invocation of the `request.el`-powered function from Rust looks like this:

```rs
    let args = vec!["http://localhost:1937/notes".to_string().into_lisp(env)?];

    let time = std::time::Instant::now();
    for _ in 0..100 {
        env.call("jiroscope-benchmark-request-el", &args)?;
    }
    let elapsed = time.elapsed();
```


# Jiroscope Test Server Benchmark

Emacs functions used for testing:

```el
; 1 elisp request + JSON conversion
(defun jiroscope-benchmark-request-el (url)
    (request-response-data
     (request url
       :parser 'json-read
       :sync t)))

; 100 requests in elisp from elisp
(defun jiroscope-benchmark-request-el-full (url)
  (dotimes (i 100)
    (jiroscope-benchmark-request-el url)))

; 100 requests in rust from elisp
(defun jiroscope-benchmark-ureq-full (url)
  (dotimes (i 100)
    (jiroscope-get-notes)))
```

## GET test server notes

| Caller | Backend | Time |
| --- | --- | --- |
| Rust | ureq | 17.778648ms |
| Rust | request.el | 2.481233749s |
| ELisp | request.el | 2.51860959s |
| ELisp | ureq | 51.629347ms |

The `request.el` results show that it's the main difference maker. Between Rust + `ureq` and
Emacs Lisp + `request.el` we can see a ~100x speed difference. This is quite a jump.

# Jira Cloud Benchmark

Emacs functions used for testing:

```el
; 1 elisp request + JSON conversion
(defun jiroscope-auth-benchmark-request-el (url auth_hash)
    (request-response-data
     (request url
       :headers '(("Authorization" . ((concat "Basic " auth_hash))))
       :parser 'json-read
       :sync t)))

; 100 requests in elisp from elisp
(defun jiroscope-auth-benchmark-request-el-full (url auth_hash)
  (dotimes (i 100)
    (jiroscope-auth-benchmark-request-el url auth_hash)))

; 100 requests in rust from elisp
(defun jiroscope-auth-benchmark-ureq-full ()
  (dotimes (i 100)
    (jiroscope-get-all-issues)))
```

## GET Jira Cloud issues

| Caller | Backend | Time |
| --- | --- | --- |
| Rust | ureq | 37.475099131s |
| Rust | request.el | 42.424662652s |
| ELisp | request.el | 42.651552689s |
| ELisp | ureq | 38.107238839s |

In a real-world scenario, calling a remote server, the differences are smaller. A difference of
4-5 seconds between `ureq` and `request.el` is not a big deal, but it still leaves `ureq` as the
strictly faster option.

# Thoughts
`ureq` is definitely faster to use in Emacs, from Rust code. The differences aren't huge in the
scale of a remote request, but they're there.

I will be sticking to having as much as my code in Rust as possible, dipping into Emacs Lisp
only for IO and exposing a simple interface. I am much more familiar with Rust than Emacs Lisp,
and it's a much more powerful, faster language overall.
