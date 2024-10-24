# myrtle_wyckoff

Super Fast Super Secret Super Hot Exchanges at Myrtle Wyckoff

- [ ] deploy contracts
- [ ] set up snapshotter
- [ ] set up orderhere
- [ ] plug all the dstack components together
- [ ] probably need to make sure i'm not being an idiot about concurrency
- [ ] use application specific secret key
- [ ] update orderbook (and related components) for 18 decimal support - need i256
- [ ] test ofc
- [ ] need some usage scripts

## Overview

There are 3 components:

- dstack app
  Responsible for most things, runs the orderbook, ingests deposits, approves settlement orders, and posts state snapshots. The dstack app can be recovered from any dstack container running this application
- mainnet Deposit Registry
  Allows users to deposit USDC and WETH into the app. In addition it allows the takers to pull settlement funds from the app when settling user orders.
- suave Checkpointer
  Posts the state snapshots to suave. These include settlement orders which are emitted as events and the encrypted inventory state which is stored in the Checkpointer contract.

More is TODO:
In the meantime https://app.excalidraw.com/l/4qzEA15BcJo/4i1VVVFdJqU
Very slightly out of date

## Run instructions

```shell
cd myrtle-wyckoff-dstack
docker build -t myrtle-wyckoff-dstack .
docker run -p 8000:8000 myrtle-wyckoff-dstack
```

# Requirements to Bring to Production (in progress)

## State Lock System

We currently don't handle the scenario where cowswap orders go unfilled. This is unsafe since we perform an inventory update assuming that the settlement will be completed successfully. This can be handled by using a state lock system that locks the portion of the inventory undergoing settlement when a settlement order is submitted, and records settlement results in a mainnet settlement registry using a post-hook. Then the dstack component would clear the settlement lock and update inventory based on the settlement result.

## Leader mechanism

## Inventory ingestion mechanism

## Probably use a data avialability layer for the inventory snapshots

## need to use a cowshed rather than the taker wallet for settlement orders

## settlement nonces should be non-sequential
