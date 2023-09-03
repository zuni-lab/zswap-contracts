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

1. Mint `ZNEAR`

```sh
# mint 100000 ZNEAR for zswap.testnet
$ near call znear.zswap.testnet mint '{"receiver_id":"zswap.testnet", "amount": "100000"}' --deposit 1 --accountId zswap.testnet
```

2. Mint `ZUSD`

```sh
# mint 20000 ZUSD for zswap.testnet
$ near call zusd.zswap.testnet mint '{"receiver_id":"zswap.testnet", "amount": "200000"}' --deposit 1 --accountId zswap.testnet
```

## Step 2: Create Pool with Factory

If testing with `ZNEAR` and `ZUSD`, you can skip this step.

1. Create a new pool for ZNEAR - ZUSD. Factory only supports creating a new pool with 2 fee levels: 0.05% and 0.3%.

```sh
$ ZSWAP_MANAGER=manager3.zswap.testnet

$ near call $ZSWAP_MANAGER create_pool \
  '{"token_0":"znear.zswap.testnet","token_1":"zusd.zswap.testnet","fee":3000}' \
  --accountId zswap.testnet --gas 300000000000000 --deposit 25

# return pool address
'2e4e39194a383739.factory3.zswap.testnet'
```

- View pool state

```sh
$ ZSWAP_FACTORY=factory3.zswap.testnet

$ near view $ZSWAP_FACTORY get_pool '{"token_0":"zusd.zswap.testnet", "token_1":"znear.zswap.testnet","fee":3000}'

{
  pool_id: '2e4e39194a383739.factory3.zswap.testnet',
  token_0: 'znear.zswap.testnet',
  token_1: 'zusd.zswap.testnet',
  fee: 3000,
  tick_spacing: 60
}
```

2. Initialize `sqrt_price` (token0 / token1)

```sh
# 1 ZNEAR = 100 ZUSD, tick ~ 46054
$ near call $ZSWAP_POOL initialize '{"sqrt_price_x96":"792281625142643375935439503360"}' --accountId zswap.testnet

# view current price
$ near view $ZSWAP_POOL get_slot_0 '{}'

{ sqrt_price_x96: '792281625142643375935439503360', tick: 46054 }

```

3. Get list of existed tokens in ZSwap Manager

```sh
$ near view $ZSWAP_MANAGER get_fungible_tokens

[ "znear.zswap.test", "zusd.zswap.testnet" ]
```

## Step 3: Mint Liquidity

1. Deposit ZNEAR into `ZswapPool`

```sh
$ ZNEAR_AMOUNT=10000000

$ near call $ZNEAR ft_transfer_call '{"receiver_id":"'$ZSWAP_POOL'", "amount":"'$ZNEAR_AMOUNT'", "msg":""}' --depositYocto 1 --gas 300000000000000 --accountId zswap.testnet
```

2. Deposit ZUSD into `ZswapPool`

```sh
$ ZUSD_AMOUNT=100000000

$ near call $ZUSD ft_transfer_call '{"receiver_id":"'$ZSWAP_POOL'", "amount":"'$ZUSD_AMOUNT'", "msg":""}' --depositYocto 1 --gas 300000000000000 --accountId zswap.testnet
```

3. Mint liquidity

- JSON schema example:

```json
{
  "params": {
    "token_0": "znear.zswap.testnet",
    "token_1": "zusd.zswap.testnet",
    "fee": 3000,
    "lower_tick": 42000,
    "upper_tick": 48000,
    "amount_0_desired": "10000000",
    "amount_1_desired": "100000000",
    "amount_0_min": "1",
    "amount_1_min": "100"
  }
}
```

- View the corresponding amount with the other.

```sh
$ SQRT_PRICE=792281625142643375935439503360

$ near view $ZSWAP_MANAGER calculate_amount_1_with_amount_0 '{"amount_0":"'$ZNEAR_AMOUNT'","sqrt_price_x96":"'$SQRT_PRICE'","lower_tick":42000,"upper_tick":48000}'

'1978918829' # returns amount_1

$ near view  $ZSWAP_MANAGER calculate_amount_0_with_amount_1 '{"amount_1":"'$ZUSD_AMOUNT'","sqrt_price_x96":"'$SQRT_PRICE'","lower_tick":42000,"upper_tick":48000}'

'505327' # returns amount_0
```

```sh
$ near call $ZSWAP_MANAGER mint '{"params":{"token_0":"'$ZNEAR'","token_1":"'$ZUSD'","fee":3000,"lower_tick":46000,"upper_tick":46100, "amount_0_desired":"'$ZNEAR_AMOUNT'","amount_1_desired":"'$ZUSD_AMOUNT'","amount_0_min":"100","amount_1_min":"100"}}' --gas 300000000000000 --accountId zswap.testnet --deposit 0.1

# Return amount_0 & amount_1
[ '505327', '100000000' ]
```

- After minting liquidity, you will get an NFT

## Step 4: Swap

This example will swap `ZNEAR` to `ZUSD`. If you need `ZNEAR` to test, following command:

```sh
$ TRADER=testz.testnet

$ near call $ZNEAR mint '{"receiver_id":"'$TRADER'", "amount": "100000"}' --deposit 1 --accountId $TRADER
```

1. Ensure register storage for trader account in FT contracts.

```sh
near call $ZUSD storage_deposit '{"account_id":"'$TRADER'"}' --deposit 1 --accountId $TRADER
```

2. Swap `ZNEAR` to `ZUSD`

```sh
$ ZNEAR_AMOUNT=100

$ SWAP_MSG='{\"swap_single\":{\"token_out\":\"'$ZUSD'\",\"fee\":3000}}'

$ near call $ZNEAR ft_transfer_call '{"receiver_id":"'$ZSWAP_MANAGER'", "amount":"'$ZNEAR_AMOUNT'", "msg":"'$SWAP_MSG'"}' --gas 300000000000000 --accountId $TRADER --depositYocto 1
```
