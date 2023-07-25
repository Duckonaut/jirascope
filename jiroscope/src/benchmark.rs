use emacs::{Env, IntoLisp, Result, Value};
use serde::{Deserialize, Serialize};

use crate::utils::{write_tuples_to_md_table, write_tuples_to_pyplot_data};

const BENCHMARK_ITERATIONS: usize = 100;

#[derive(Serialize, Deserialize)]
struct TestTree {
    value: i64,
    left: Option<Box<TestTree>>,
    right: Option<Box<TestTree>>,
}

#[emacs::defun]
fn full_data_buffer_benchmark(env: &Env) -> Result<Value> {
    let increments = &[1, 10, 100, 1000, 10000, 1000000];
    let mut rows_verbose = vec![];
    let mut rows_micro = vec![];

    for inc in increments {
        let timer_a = std::time::Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            let _ = env.call(
                "jiroscope-benchmark-rust-lisp-untouched",
                [inc.into_lisp(env)?],
            )?;
        }
        let elapsed = timer_a.elapsed();
        let time_a = format!("{:?}", elapsed);
        let time_a_micro = elapsed.as_micros();

        let timer_b = std::time::Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            let _ = env.call(
                "jiroscope-benchmark-lisp-rust-untouched",
                [inc.into_lisp(env)?],
            )?;
        }
        let elapsed = timer_b.elapsed();
        let time_b = format!("{:?}", elapsed);
        let time_b_micro = elapsed.as_micros();

        let timer_c = std::time::Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            let _ = env.call(
                "jiroscope-benchmark-rust-lisp-touched",
                [inc.into_lisp(env)?],
            )?;
        }
        let elapsed = timer_c.elapsed();
        let time_c = format!("{:?}", elapsed);
        let time_c_micro = elapsed.as_micros();

        let timer_d = std::time::Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            let _ = env.call(
                "jiroscope-benchmark-lisp-rust-touched",
                [inc.into_lisp(env)?],
            )?;
        }
        let elapsed = timer_d.elapsed();
        let time_d = format!("{:?}", elapsed);
        let time_d_micro = elapsed.as_micros();

        let timer_e = std::time::Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            let _ = env.call(
                "jiroscope-benchmark-rust-back-and-forth",
                [inc.into_lisp(env)?],
            )?;
        }
        let elapsed = timer_e.elapsed();
        let time_e = format!("{:?}", elapsed);
        let time_e_micro = elapsed.as_micros();

        let timer_f = std::time::Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            let _ = env.call(
                "jiroscope-benchmark-rust-normal-call",
                [inc.into_lisp(env)?],
            )?;
        }
        let elapsed = timer_f.elapsed();
        let time_f = format!("{:?}", elapsed);
        let time_f_micro = elapsed.as_micros();

        rows_verbose.push((inc, time_a, time_b, time_c, time_d, time_e, time_f));

        rows_micro.push((
            inc,
            time_a_micro,
            time_b_micro,
            time_c_micro,
            time_d_micro,
            time_e_micro,
            time_f_micro,
        ));
    }

    let mut file = std::fs::File::create("jiroscope-benchmark.md")?;

    write_tuples_to_md_table(
        &mut file,
        &[
            "Buffer Size (bytes)",
            "From Rust to Lisp (no read)",
            "From Lisp to Rust (no read)",
            "From Rust to Lisp (read)",
            "From Lisp to Rust (read)",
            "Back and Forth",
            "Normal Call",
        ],
        &rows_verbose,
    )?;

    let mut file = std::fs::File::create("jiroscope-benchmark-data.py")?;

    write_tuples_to_pyplot_data(
        &mut file,
        &[
            "Buffer Size (bytes)",
            "From Rust to Lisp (no read)",
            "From Lisp to Rust (no read)",
            "From Rust to Lisp (read)",
            "From Lisp to Rust (read)",
            "Back and Forth",
            "Normal Call",
        ],
        &rows_micro,
    )?;

    ().into_lisp(env)
}

#[emacs::defun]
fn full_json_benchmark(env: &Env) -> Result<Value> {
    let depths = &[1, 2, 4, 6, 8, 10];
    let mut file = std::fs::File::create("jiroscope-benchmark.md")?;
    let mut rows_verbose = vec![];
    let mut rows_micro = vec![];

    for depth in depths {
        let json = gen_test_tree_string(*depth);

        let timer_a = std::time::Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            let _ = env.call(
                "jiroscope-benchmark-rust-deserialize-json-no-navigate",
                [json.clone().into_lisp(env)?],
            )?;
        }
        let elapsed = timer_a.elapsed();
        let time_a = format!("{:?}", elapsed);
        let time_a_micro = elapsed.as_micros();

        let timer_b = std::time::Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            let _ = env.call(
                "jiroscope-benchmark-lisp-deserialize-json-no-navigate",
                [json.clone().into_lisp(env)?],
            )?;
        }
        let elapsed = timer_b.elapsed();
        let time_b = format!("{:?}", elapsed);
        let time_b_micro = elapsed.as_micros();

        let timer_c = std::time::Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            let _ = env.call(
                "jiroscope-benchmark-rust-deserialize-json",
                [json.clone().into_lisp(env)?],
            )?;
        }
        let elapsed = timer_c.elapsed();
        let time_c = format!("{:?}", elapsed);
        let time_c_micro = elapsed.as_micros();

        let timer_d = std::time::Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            let _ = env.call(
                "jiroscope-benchmark-lisp-deserialize-json",
                [json.clone().into_lisp(env)?],
            )?;
        }
        let elapsed = timer_d.elapsed();
        let time_d = format!("{:?}", elapsed);
        let time_d_micro = elapsed.as_micros();

        rows_verbose.push((depth, time_a, time_b, time_c, time_d));
        rows_micro.push((
            depth,
            time_a_micro,
            time_b_micro,
            time_c_micro,
            time_d_micro,
        ));
    }

    write_tuples_to_md_table(
        &mut file,
        &[
            "Binary tree depth",
            "Deserialization (Rust, no navigation)",
            "Deserialization (Lisp, no navigation)",
            "Deserialization (Rust)",
            "Deserialization (Lisp)",
        ],
        &rows_verbose,
    )?;

    let mut file = std::fs::File::create("jiroscope-benchmark-plot-data.py")?;
    write_tuples_to_pyplot_data(
        &mut file,
        &[
            "Binary tree depth",
            "Deserialization (Rust, no navigation)",
            "Deserialization (Lisp, no navigation)",
            "Deserialization (Rust)",
            "Deserialization (Lisp)",
        ],
        &rows_verbose,
    )?;

    ().into_lisp(env)
}

fn gen_test_tree_string(depth: usize) -> String {
    let mut tree = TestTree {
        value: 0,
        left: None,
        right: None,
    };

    let mut nodes = vec![&mut tree];

    for i in 0..depth {
        let mut new_nodes = vec![];

        for node in nodes {
            node.left = Some(Box::new(TestTree {
                value: i as i64,
                left: None,
                right: None,
            }));
            node.right = Some(Box::new(TestTree {
                value: i as i64,
                left: None,
                right: None,
            }));

            new_nodes.push(node.left.as_deref_mut().unwrap());
            new_nodes.push(node.right.as_deref_mut().unwrap());
        }

        nodes = new_nodes;
    }

    serde_json::to_string(&tree).unwrap()
}

mod rust {
    use emacs::{Env, IntoLisp, Result, Value};

    use super::TestTree;

    #[emacs::defun]
    fn lisp_untouched(env: &Env, inc: i64) -> Result<Value> {
        let mut buffer = Vec::with_capacity(inc as usize);
        buffer.resize(inc as usize, b'R');
        let s = unsafe { String::from_utf8_unchecked(buffer) }; // fine because we know it's all
                                                                // ASCII 'R's

        env.call("jiroscope-lisp-get-no-read", [s.into_lisp(env)?])
    }

    #[emacs::defun]
    fn get_no_read(env: &Env, _: String) -> Result<Value> {
        ().into_lisp(env)
    }

    #[emacs::defun]
    fn lisp_touched(env: &Env, inc: i64) -> Result<Value> {
        let mut buffer = Vec::with_capacity(inc as usize);
        buffer.resize(inc as usize, b'R');
        let s = unsafe { String::from_utf8_unchecked(buffer) }; // fine because we know it's all
                                                                // ASCII 'R's

        env.call("jiroscope-lisp-get-read", [s.into_lisp(env)?])
    }

    #[emacs::defun]
    fn get_read(env: &Env, buffer: String) -> Result<Value> {
        let b = buffer.as_bytes();
        let mut sum = 0;
        for b in b {
            sum += *b as i64;
        }
        sum.into_lisp(env)
    }

    #[emacs::defun]
    fn back_and_forth(env: &Env, inc: i64) -> Result<Value> {
        let mut buffer = Vec::with_capacity(inc as usize);
        buffer.resize(inc as usize, b'R');
        let s = unsafe { String::from_utf8_unchecked(buffer) }; // fine because we know it's all

        env.call("jiroscope-benchmark-rust-get-read", [s.into_lisp(env)?])
    }

    #[emacs::defun]
    fn normal_call(env: &Env, inc: i64) -> Result<Value> {
        let mut buffer = Vec::with_capacity(inc as usize);
        buffer.resize(inc as usize, b'R');
        let s = unsafe { String::from_utf8_unchecked(buffer) }; // fine because we know it's all

        get_read(env, s)
    }

    #[emacs::defun]
    fn deserialize_json_no_navigate(env: &Env, s: String) -> Result<Value> {
        let v: TestTree = serde_json::from_str(&s)?;

        v.value.into_lisp(env)
    }

    #[emacs::defun]
    fn deserialize_json(env: &Env, s: String) -> Result<Value> {
        let v: TestTree = serde_json::from_str(&s)?;

        // return value of the deepest, leftmost node
        let mut node = &v;

        while let Some(left) = &node.left {
            node = left;
        }

        node.value.into_lisp(env)
    }
}
