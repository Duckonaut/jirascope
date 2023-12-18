build:
    cargo build
    cp target/debug/libjirascope_dyn.so jirascope/jirascope-dyn.so

testing:
    cargo build --features full
