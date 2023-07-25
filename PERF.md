# Performance testing
When making an Emacs package, I wanted to make one using the Emacs module system, rather than
just in Emacs Lisp. This allows for using more powerful languages, like C++ or Rust
(or anything that can bind to C functions).

\subsection{Rust in Emacs}
The Rust crate ecosystem seemed to have an interesting crate, called just \verb|emacs| in the
[crates.io](https://crates.io) registry. It creates Rust <-> Emacs bindings in a simple way,
and since Rust statically links everything by default, this would let me use any other crates
I wanted while still creating a single binary without major tweaks to the default \verb|cargo| build
system.

I needed to test the \verb|emacs| crate performance from two angles. Call delay, introduced by calling a Rust function using the Emacs Lisp environment,
and argument translation, which happens when a Rust function is supposed to receive an Emacs value (eg. a \verb|String|)

I also wanted to see if the Rust optimization played a role in the results, so I tested passing a buffer and reading it, and also passing a buffer and ignoring it.

There were 6 benchmarks in this round:
- \verb|String| created in Rust and passed to Lisp, ignored
- \verb|String| created in Lisp and passed to Rust, ignored
- \verb|String| created in Rust and passed to Lisp, bytes summed up
- \verb|String| created in Lisp and passed to Rust, bytes summed up
- \verb|String| created in Rust and passed to Rust, but calling the Rust function using the Lisp environment
- \verb|String| created in Rust and passed to Rust the normal way.

| Buffer Size (bytes) | From Rust to Lisp (no read) | From Lisp to Rust (no read) | From Rust to Lisp (read) | From Lisp to Rust (read) | Back and Forth | Normal Call | 
| --- | --- | --- | --- | --- | --- | --- |
| 1 |279.608µs |281.577µs |771.036µs |292.623µs |334.178µs |134.718µs |
| 10 |263.55µs |307.899µs |810.517µs |53.296µs |66.175µs |24.365µs |
| 100 |71.413µs |55.021µs |3.310196ms |69.795µs |89.37µs |28.86µs |
| 1000 |199.149µs |99.099µs |30.377047ms |127.615µs |233.382µs |49.722µs |
| 10000 |1.460132ms |491.172µs |283.645887ms |746.92µs |1.722699ms |276.252µs |
| 1000000 |152.463153ms |35.399373ms |29.886151028s |77.347301ms |164.289196ms |26.511211ms |

I also needed to test JSON deserialization performance to a form I could navigate around, since
Jira's API uses that as the response format. For that, I created a Rust function that would
generate a binary tree of depth N, then save it to JSON. Then, the string form would get passed
to two functions: a Rust one, which deserialized it using \verb|serde|, and a Lisp one, which
deserialized it using the builtin \verb|json| package to a hash table.

Afterwards, I tested with similar functions, but ones which navigate the deserialized tree.
After getting an object representation, they get the deepest, leftmost node value.

The performance comparison is as follows:

| Binary tree depth | Deserialization (Rust, no navigation) | Deserialization (Lisp, no navigation) | Deserialization (Rust) | Deserialization (Lisp) | 
| --- | --- | --- | --- | --- |
| 1 | 444.339µs | 5.771178ms | 81.39µs | 3.193572ms |
| 2 | 124.788µs | 5.956935ms | 131.717µs | 5.914346ms |
| 4 | 467.986µs | 22.198033ms | 458.577µs | 22.003751ms |
| 6 | 1.781021ms | 119.83089ms | 1.552814ms | 70.519021ms |
| 8 | 6.549073ms | 371.426712ms | 6.387685ms | 356.678699ms |
| 10 | 26.734809ms | 1.454119205s | 27.057188ms | 1.405706409s |

\subsection{Testing HTTP connectivity}
As the project requires internet connectivity, I had two choices. I could either use a package
already offerred in Emacs, like \verb|url.el| or \verb|request.el| which are very popular, or introduce a
Rust dependency. \verb|request.el| has a pretty nice, relatively high level API. The most comparable
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
that it is. This is why a blocking request library like \verb|ureq| seemed like a safe choice.

But, worried about the performance penalty of translating objects between Rust and Emacs Lisp
and the general delay due to the boundary of Emacs Lisp and Rust, I wanted to benchmark
`request.el` against \verb|ureq|. There are four "tiers" of the integration I tested:

- **Exclusively Emacs Lisp**: The request is handled by \verb|request.el|, and the processing and
  looping of it is done in Emacs Lisp.
- **Emacs Lisp using simple Rust functions**: The request and decoding of the HTTP request is
  done in Rust code, in simple single-request functions. The loop is done in Emacs Lisp.
- **Avoiding Emacs Lisp**: As much as possible is done from Rust, without crossing over the
  Emacs Lisp boundary.
- **Calling \verb|request.el| from Rust**: This one is mainly a curiosity. It doesn't *really* make
  sense to use in practice for anything I can think of, but it should show the performance
  difference between the two libraries, ignoring the Emacs Lisp interpreter performance penalty
  where possible.

There are also two testing scenarios I ran through. A local test server (already on hand from
initial development of the package), and the remote Jira Cloud server. This would tell me
the "raw" performance of the approaches, and later show how much that actually matters in
a real world scenario. Both are simple GET requests, since the type of request and size of the
payload do not carry any real extra information to differentiate the approaches.

To be representative of the end goal, everything was called from Rust code, and the time
measurement done using the Rust standard \verb|std::time| types.

An example invocation of the \verb|request.el|-powered function from Rust looks like this:

```rs
    let args = vec!["http://localhost:1937/notes".to_string().into_lisp(env)?];

    let time = std::time::Instant::now();
    for _ in 0..100 {
        env.call("jiroscope-benchmark-request-el", &args)?;
    }
    let elapsed = time.elapsed();
```


\subsubsection{Jiroscope Test Server Benchmark}

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

\subsubsubsection{GET test server notes}

| Caller | Backend | Time |
| --- | --- | --- |
| Rust |ureq |120.670281ms |
| Rust |request.el |2.832791099s |
| ELisp |request.el |3.10754881s |
| ELisp |ureq |31.26165ms |

The \verb|request.el| results show that it's the main difference maker. Between Rust + \verb|ureq| and
Emacs Lisp + \verb|request.el| we can see a ~100x speed difference. This is quite a jump.

\subsubsection{Jira Cloud Benchmark}

Emacs functions used for testing:

```el
; 1 elisp request + JSON conversion
(defun jiroscope-auth-benchmark-request-el (url auth_hash)
    (request-response-data
        (request url
            :parser 'json-read
            :headers `(("Authorization" . ,(concat "Basic " auth_hash)) ("Content-Type" . "application/json"))
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

\subsubsubsection{GET Jira Cloud issues}
| Caller | Backend | Time |
| --- | --- | --- |
| Rust |ureq |42.05248701s |
| Rust |request.el |46.637762647s |
| ELisp |request.el |46.536621777s |
| ELisp |ureq |41.336404793s |

In a real-world scenario, calling a remote server, the differences are smaller. A difference of
6-7 seconds between \verb|ureq| and \verb|request.el| is not a big deal, but it still leaves \verb|ureq| as the
strictly faster option.

\subsection{Thoughts}
`ureq` is definitely faster to use in Emacs, from Rust code. The differences aren't huge in the
scale of a remote request, but they're there.

I will be sticking to having as much as my code in Rust as possible, dipping into Emacs Lisp
only for IO and exposing a simple interface. I am much more familiar with Rust than Emacs Lisp,
and it's a much more powerful, faster language overall.


