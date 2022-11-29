#!/bin/bash
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./bin/ --no-typescript --browser --target web ./target/wasm32-unknown-unknown/release/ant-sim.wasm
