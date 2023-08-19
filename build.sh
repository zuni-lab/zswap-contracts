#!/bin/sh

echo ">> Building Zswap contract"

rustup target add wasm32-unknown-unknown
cargo build --all --target wasm32-unknown-unknown --release

cp target/wasm32-unknown-unknown/release/zswap_pool.wasm res/
cp target/wasm32-unknown-unknown/release/zswap_manager.wasm res/
cp target/wasm32-unknown-unknown/release/zswap_factory.wasm res/
