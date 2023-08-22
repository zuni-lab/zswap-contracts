#!/bin/sh

echo ">> Building Zswap contracts..."

rustup target add wasm32-unknown-unknown
cargo build --package zswap-pool --target wasm32-unknown-unknown --release
cargo build --package zswap-manager --target wasm32-unknown-unknown --release
cargo build --package zswap-factory --target wasm32-unknown-unknown --release

cp target/wasm32-unknown-unknown/release/zswap_pool.wasm res/
cp target/wasm32-unknown-unknown/release/zswap_manager.wasm res/
cp target/wasm32-unknown-unknown/release/zswap_factory.wasm res/
