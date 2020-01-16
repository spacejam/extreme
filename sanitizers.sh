#!/bin/bash
set -eo pipefail

echo "asan"
cargo clean
export RUSTFLAGS="-Z sanitizer=address"
export ASAN_OPTIONS="detect_odr_violation=0"
cargo +nightly build --tests --target x86_64-unknown-linux-gnu
$(ls ./target/x86_64-unknown-linux-gnu/debug/smoke* | grep -v '\.d')
unset ASAN_OPTIONS

echo "lsan"
cargo clean
export RUSTFLAGS="-Z sanitizer=leak"
cargo +nightly build --tests --target x86_64-unknown-linux-gnu
$(ls ./target/x86_64-unknown-linux-gnu/debug/smoke* | grep -v '\.d')

echo "tsan"
cargo clean
export RUSTFLAGS="-Z sanitizer=thread"
export TSAN_OPTIONS=suppressions=tsan_suppressions.txt
cargo +nightly build --tests --target x86_64-unknown-linux-gnu
$(ls ./target/x86_64-unknown-linux-gnu/debug/smoke* | grep -v '\.d')
