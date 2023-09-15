build:
    cargo build --release
    cp target/release/libjiroscope_dyn.so jiroscope/jiroscope-dyn.so

testing:
    cargo build --features full
