# myrtle_wyckoff

Super Fast Super Secret Super Hot Exchanges at Myrtle Wyckoff

- [ ] deploy contracts
- [ ] contract needs to to support erc-1271 signatures via hooks
- [ ] set up settlement orders (cowswap orders)
- [ ] set up depositor
- [ ] set up snapshotter
- [ ] use application specific key
- [ ] update orderbook for 18 decimal support

## Run instructions

```shell
cd myrtle-wyckoff-dstack
docker build -t myrtle-wyckoff-dstack .
docker run -p 8000:8000 myrtle-wyckoff-dstack
```

# Requirements to Bring to Production

## State Lock System

We currently don't handle the scenario where cowswap orders go unfilled. This is unsafe since we perform an inventory update assuming that the settlement will be completed successfully. This can be handled by using a state lock system that locks the portion of the inventory undergoing settlement when a settlement order is submitted, and records settlement results in a mainnet settlement registry using a post-hook. Then the dstack component would clear the settlement lock and update inventory based on the settlement result.

## Leader mechanism

## Inventory ingestion mechanism

## Probably don't use blobs

## need to use a cowshed rather than the taker wallet for settlement orders

## settlement nonces should be non-sequential
