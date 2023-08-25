#!/bin/sh

ZSWAP=zswap.testnet
MANAGER=manager.zswap.testnet
FACTORY=factory2.zswap.testnet
POOL="2e4e39194a383739.factory2.zswap.testnet"
ZNEAR=znear.zswap.testnet
ZUSD=zusd.zswap.testnet
POOL_FEE=3000

# ===== UPDATE POOL =====
# 1. Delete old pool account
echo -e "\nDeleting old pool account..."
near delete $POOL $FACTORY

# 2. Re-create pool account
echo -e "\nRe-create pool account..."
near create-account 2e4e39194a383739.factory2.zswap.testnet --masterAccount $FACTORY --initialBalance 50

3. Deploy pool contract
echo -e "\nDeploying pool contract..."
near deploy --wasmFile ../res/zswap_pool.wasm --accountId $POOL

# 4. Initialize pool contract
echo -e "\nInitializing pool contract..."
near call $POOL new '{"token_0":"'$ZNEAR'","token_1":"'$ZUSD'","fee":'$POOL_FEE',"tick_spacing":60}' --accountId $POOL --gas 300000000000000

# NEAR/ZUSD = 100, tick ~ 46054
near call $POOL initialize '{"sqrt_price_x96":"792281625142643375935439503360"}' --accountId $ZSWAP

# 5. Add liquidity to pool
ZNEAR_AMOUNT=1000
ZUSD_AMOUNT=100000
echo -e "\nAdding liquidity to pool..."

near call $ZNEAR ft_transfer_call '{"receiver_id":"'$POOL'", "amount":"'$ZNEAR_AMOUNT'", "msg":""}' --depositYocto 1 --gas 300000000000000 --accountId $ZSWAP
near call $ZUSD ft_transfer_call '{"receiver_id":"'$POOL'", "amount":"'$ZUSD_AMOUNT'", "msg":""}' --depositYocto 1 --gas 300000000000000 --accountId $ZSWAP

near call $MANAGER mint '{"params":{"token_0":"'$ZNEAR'","token_1":"'$ZUSD'","fee":'$POOL_FEE',"lower_tick":42000,"upper_tick":48000, "amount_0_desired":"'$ZNEAR_AMOUNT'","amount_1_desired":"'$ZUSD_AMOUNT'","amount_0_min":"100","amount_1_min":"100"}}' --gas 300000000000000 --accountId $ZSWAP --deposit 0.1