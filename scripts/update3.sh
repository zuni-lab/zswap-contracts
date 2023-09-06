#!/bin/sh

ZSWAP=zswap.testnet
MANAGER=manager3.zswap.testnet
FACTORY=factory3.zswap.testnet
POOL="2e4e39194a383739.factory3.zswap.testnet"
ZNEAR=znear.zswap.testnet
ZUSD=zusd.zswap.testnet
POOL_FEE=3000
MAX_GAS=300000000000000

# ===== UPDATE FACTORY =====
# 1. Delete pool code in factory
echo ""
echo "Deleting pool code in factory..."
near call $FACTORY remove_stored_contract '{}' --accountId $FACTORY --gas $MAX_GAS

# 2. Delete old factory
echo ""
echo "Deleting old factory..."
near delete $FACTORY $ZSWAP

# 3. Re-create factory
echo ""
echo "Creating factory account again..."
near create-account $FACTORY --masterAccount $ZSWAP --initialBalance 120

# 4. Deploy & Init factory contract
echo "Deploying & Initializing factory contract..."
near deploy --wasmFile ./res/zswap_factory.wasm --accountId $FACTORY --initFunction new --initArgs '{}' --initGas $MAX_GAS

# ===== DELETE POOL =====
# 1. Delete old pool account
echo ""
echo "Deleting old pool account..."
near delete $POOL $FACTORY

# ===== UPDATE MANAGER =====
# 1. Delete old manager
echo ""
echo "Deleting old manager..."
near delete $MANAGER $ZSWAP

# 2. Re-create manager
echo ""
echo "Creating manager account again..."
near create-account $MANAGER --masterAccount $ZSWAP --initialBalance 120

# 3. Deploy & Init manager contract
echo ""
echo "Deploying & Initializing manager contract..."
near deploy --wasmFile ./res/zswap_manager.wasm --accountId $MANAGER --initFunction new --initArgs '{"factory":"'$FACTORY'"}' --initGas $MAX_GAS

# 4. Re-create pool
echo ""
echo "Re-creating pool..."
near call $MANAGER create_pool '{"token_0":"'$ZNEAR'","token_1":"'$ZUSD'","fee":'$POOL_FEE',"sqrt_price_x96":"792281625142643375935439503360"}' --accountId $ZSWAP --gas $MAX_GAS --deposit 25

