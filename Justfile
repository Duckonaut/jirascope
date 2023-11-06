build:
    cargo build
    cp target/debug/libjiroscope_dyn.so jiroscope/jiroscope-dyn.so

testing:
    cargo build --features full
