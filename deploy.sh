#!/bin/sh

./build.sh

if [ $? -ne 0 ]; then
  echo ">> Error building Zswap contracts!"
  exit 1
fi

echo ">> Deploying Zswap contracts..."

# https://docs.near.org/tools/near-cli#near-dev-deploy
near dev-deploy --wasmFile ./res/zswap_pool.wasm
near dev-deploy --wasmFile ./res/zswap_manager.wasm
near dev-deploy --wasmFile ./res/zswap_factory.wasm
