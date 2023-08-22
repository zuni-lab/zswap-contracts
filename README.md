# ZSwap

## Step 0: Create a new account

0. Install near-cli

```sh
$ npm install --global near-cli
```

1. Create a new account on [MyNearWallet testnet](https://testnet.mynearwallet.com/)

2. Authorize NEAR CLI, following the commands it gives you:

```sh
# login to zswap.testnet account
$ near login --walletUrl https://testnet.mynearwallet.com
```

3. Create a subaccount (optional):

```sh
# create `sub.zswap.testnet` for `zswap.testnet` with 20 NEAR
$ near create-account sub.zswap.testnet --masterAccount zswap.testnet --initialBalance 20
```

## Step 1: Mint tokens

1. Mint `WNEAR`

```sh
# mint 100000 WNEAR for zswap.testnet
$ near call wnear.zswap.testnet mint '{"receiver_id":"zswap.testnet", "amount": "100000"}' --deposit 1 --accountId zswap.testnet
```

2. Mint `ZUSD`

```sh
# mint 20000 ZUSD for zswap.testnet
$ near call zusd.zswap.testnet mint '{"receiver_id":"zswap.testnet", "amount": "200000"}' --deposit 1 --accountId zswap.testnet
```

## Step 2: Create Pool with Facotry

If testing with `ZNEAR` and `ZUSD`, you can skip this step.

1. Create a new pool for WNEAR - ZUSD. Factory only supports creating a new pool with 2 fee levels: 0.05% and 0.3%.

```sh
$ near call factory.zswap.testnet create_pool \
  '{"token_0":"wnear.zswap.testnet","token_1":"zusd.zswap.testnet","fee":3000}' \
  --accountId zswap.testnet --gas 300000000000000 --deposit 25

# return pool address
'b0f160b912d575db.factory.zswap.testnet'
```

- View pool state

```sh
$ near view factory.zswap.testnet get_pool '{"token_0":"zusd.zswap.testnet", "token_1":"wnear.zswap.testnet","fee":3000}'
```

2. Register storage for `ZswapPool` in FT contracts.

```sh
WNEAR=wnear.zswap.testnet
ZUSD=zusd.zswap.testnet
ZSWAP_POOL=b0f160b912d575db.factory.zswap.testnet

# register storage for `ZswapPool` in `WNEAR`
near call $WNEAR storage_deposit '{"account_id":"'$ZSWAP_POOL'"}' --deposit 1 --accountId zswap.testnet

# register storage for `ZswapPool` in `ZUSD`
near call $ZUSD storage_deposit '{"account_id":"'$ZSWAP_POOL'"}' --deposit 1 --accountId zswap.testnet
```

3. Initialize `sqrt_price`

```sh
# 1 WNEAR = 100 ZUSD, tick ~ 46054
near call $ZSWAP_POOL initialize '{"sqrt_price_x96":"792281625142643375935439503360"}' --accountId zswap.testnet
```

## Step 3: Mint Liquidity

1. Deposit WNEAR into `ZswapManager`

```sh
WNEAR_AMOUNT=1000

near call $WNEAR ft_transfer_call '{"receiver_id":"'$ZSWAP_POOL'", "amount":"'$WNEAR_AMOUNT'", "msg":""}' --depositYocto 1 --gas 300000000000000 --accountId zswap.testnet
```

2. Deposit ZUSD into `ZswapManager`

```sh
ZUSD_AMOUNT=100000

near call $ZUSD ft_transfer_call '{"receiver_id":"'$ZSWAP_POOL'", "amount":"'$ZUSD_AMOUNT'", "msg":""}' --depositYocto 1 --gas 300000000000000 --accountId zswap.testnet
```

3. Mint liquidity

- JSON schema example:

```json
{
  "params": {
    "token_0": "wnear.zswap.testnet",
    "token_1": "zusd.zswap.testnet",
    "fee": 3000,
    "lower_tick": 46000,
    "upper_tick": 46100,
    "amount_0_desired": "10",
    "amount_1_desired": "500",
    "amount_0_min": "1",
    "amount_1_min": "100"
  }
}
```

```sh
ZSWAP_MANAGER=zswap-manager.testnet

$ near call $ZSWAP_MANAGER mint '{"params":{"token_0":"'$WNEAR'","token_1":"'$ZUSD'","fee":3000,"lower_tick":46000,"upper_tick":46100, "amount_0_desired":"10","amount_1_desired":"500","amount_0_min":"1","amount_1_min":"100"}}' --gas 300000000000000 --accountId zswap.testnet

# Return amount_0 & amount_1
[ '6', '500' ]
```
