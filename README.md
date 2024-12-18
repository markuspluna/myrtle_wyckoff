# myrtle_wyckoff

Super Fast Super Secret Super Hot Exchanges at Myrtle Wyckoff

- [ ] deploy contracts
- [ ] need gas cost estimation
- [ ] add dstack framework
- [ ] plug all the dstack components together
- [ ] probably need to make sure i'm not being an idiot about concurrency
- [ ] test ofc
- [ ] need some usage scripts

## Overview

There are 3 components:

- dstack app
  Responsible for most things, runs the orderbook, ingests deposits, approves settlement orders, and posts state snapshots. The dstack app can be recovered from any dstack container running this application
- base chain Deposit Registry
  Allows users to deposit USDC and WETH into the app. In addition it allows the takers to pull settlement funds from the app when settling user orders.
- suave Checkpointer
  Posts the state snapshots to suave. These include settlement orders which are emitted as events and the encrypted inventory state which is stored in the Checkpointer contract. Switching to a different da layer may be desireable.

More is TODO:
In the meantime https://app.excalidraw.com/l/4qzEA15BcJo/4i1VVVFdJqU
Very slightly out of date

## Run instructions

```shell
cd myrtle-wyckoff-dstack
docker build -t myrtle-wyckoff-dstack .
docker run -p 8000:8000 myrtle-wyckoff-dstack
```

# Create an encrypted volume

docker volume create --driver local \
 --opt type=luks \
 --opt device=/path/to/encrypted/storage \
 --name myrtle_encrypted_data

# Run the container

docker run -d \
 -p 8000:8000 \
 -e RPC_URL=https://your-rpc-endpoint \
 -e ENCRYPTION_KEY=your-32-byte-hex-key \
 -e DSTACK_SECRET=your-dstack-secret \
 -v myrtle_encrypted_data:/app/encrypted_data \
 myrtle-wyckoff-dstack

# Outstanding work for Demo MVP (CME ready)

Rather than releasing Myrtle Wyckoff as a standalone demo of t+ we're going to release it as the rails for CME: The Pit. Using the game as a demo should be a significantly more interesting way of communicating what t+ is actually used for. And why it's interesting. There's not much work that is a "waste" here, I think it's mostly the settlement stuff which we might want to think about re-architecting. But the cowswap integration is really good for the demo.

Work required to get to a demoable point is roughly outlined below but will be described in more detail in issues on this repo.

## Dstack Work

### Quote Verification

Tdeps dstack version doesn't actually support quote verification. We need to add that.

##### Required Work

- [] Add quote verification to dstack
- Considerations: Flashbots might finally release a full dstack version which would make this a waste of time.

### Leader Management (Dstack work)

The leader is a semi-trusted party since it posts state snapshots and approves settlement orders and withdrawals. Since this is TDX it's probably fine, but we probably want a security council to manage the leader. Maybe just security council based view changes as well.

##### Required Work

-[] Confirm wow do you even distinguish btw various dstack containers? With the public key associated with them right?

I believe this means settlements will need both a signature from the leader and the shared secret.

## Base Chain Work

### Security Council & Leader

The checkpointer contract needs to track a security council that is able to change the leader 3/5 multisig is probably fine. Leader key should be generated inside the TDX env after the dstack container is onboarded and add a endpoint that reports the public key.

##### Required Work

- [] Add a security council multisig to the checkpointer contract.
- [] Add a update_leader function to the checkpointer contract that can be called by the security council multisig.
- [] Add an endpoint to myrtle wyckoff that reports the leader's public key.
- [] Require that state checkpoints are signed by the leader kettle's key.
- [] Security council can probably just be like a gnosis safe or something but we should figure out which solution is best.

### Withdrawal Management

We need to add withdrawals. These must be posted as part of a state snapshot and processed after the snapshot is posted so that we can ensure that the box turning off won't break everything.

Withdrawals can't be processed if the user has any liabilities.

##### Required Work

- [] Add a withdraw function to the deposit registry contract, it must validate a signature from dstack shared secret and from the leader kettle's key. It also requires replay protection which can probably be achieved through a standard user-specific nonce system.
- [] Add a withdraw function to the dstack app. This function must validate that the user has no liabilities and sufficient funds to cover the withdrawal. After that point it should structure and sign a withdrawal transaction and add it to a withdrawal request queue which is posted as part of the next state checkpoint. The queue can look a lot like the settlement order queue. We can also go ahead and deduct the withdrawal from the user's inventory balance at this point. We should emit the approved withdrawal as an event and have some service actually submit it.

### Settlements

#### Settlement Order Nonce Management

We can't use sequential nonces for settlement orders because we're probably going to be handling a lot of them at once during the settlement period. We could just store used signatures on the deposit registry and check that the sig is new, this is kinda inefficient but it's probably fine. There also might be something here where the signature is valid as long as it's the first one they've seen for that pair on a given block. Combine this with timestamp verification and it's actually pretty secure, especially since settlement orders are encrypted with the admin (or leaders) public key.

##### Required Work

- [] Add a better nonce management system to the deposit registry contract.

## Multi-Chain Deposit Support

We want to add support for at least base. This shouldn't be too hard, it just means we need to add a new deposit vault on base.

### Deposits

#### Multi-Chain Deposit Support

Deposit registry needs to be deployed on multiple chains.

##### Required Work

- [] Multi-chain replay protection needs to be considered.

#### Multi-Asset Deposit Support

We want to accept all kinds of deposits.

##### Required Work

- [] Update deposit registry contract to support deposits of any kind.
- [] We're going to need to consider chain specific deposit ingestion here in case tokens have different contract addresses across chains

### Access to Chain Data

We can use an RPC for chain data but maybe it's preferable to use a light client I think. Something to consider.

## Myrtle Wyckoff Work

### Executor Integration

We need to add an executor role, this is a key associated with another dstack program (the executor) that submits trades on behalf of users. The public key associated with the executor will be passed into myrtle wyckoff by the security council. It could also just be an initialization argument.

##### Required Work

- [] Add an executor role to the myrtle wyckoff app. Public key stored in the encrypted data volume.

### Trade Permissioning

The executor will need to be able to trade on behalf of users. Users are not allowed to trade on behalf of themselves.

##### Required Work

- [] Update trade submission functions to require a signature from the executor's key.

### Multi-Asset & Orderbook Support

We need to tweak the inventory and orderbook management system to support multiple assets.

##### Required Work

- [] Update inventory management system to support multiple assets.
- [] Update orderbook management system to support multiple orderbooks with different assets.
- [] Security council is the one that adds new orderbooks.
- [] We'll need some sort of orderbook directory

### Margin Engine

Margin engine needs to be designed. Big TBD here. Markus to talk to Finn and other risk background people.

We're not going to write a solver algo at least at this point so it might be easiest to just use the cowswap quoter as our oracle for EVM based assets and use the jupiter quoter for solana based assets. Need to check on the limitations of these.

Might be interesting to use the new proposed mango risk engine here

### Settlement

#### Settlement Order Ingestion

Currently Myrtle Wyckoff just assumes settlement was completed successfully. We need to add a mechanism for ingesting settlement registry events (or updates) and updating the user's inventory accordingly. This likely will require us to utilize the cowshed system. As well as a settlement order registry on the deposit registry contract. One of the complications here is figuring out how to handle surplus' for batch orders, not terribly complicated but it's something we need to consider.

##### Required Work

- [] Explore the best way to use cowshed for this. There are plenty of examples of how to do this.

#### Settler Role Management

We need to add a settler role, this is a key set by the security council. A signature from the settler is requires to submit settlement orders.

##### Required Work

- [] Add functions that allow the security council to set and update the settler key.
- [] Add signature verification to the settlement order submission functions.

#### Settlement Order Encryption

Settlement orders need to be encrypted with the settler's public key.

##### Required Work

- [] Encrypt settlement orders with the settler's public key. TODO: I have no idea how to do this but I do know it's possible.

#### Inventory Shuttling

The settler needs to be able to move USDC between vaults on different chains via across bridging orders.

##### Required Work

- [] Add a myrtle wyckoff function that can be called by the settler to submit an across order on behalf of the deposit vault contract.
- [] Add a deposit registry function that can be called to initiate a bridging order on accross. There might be a creative way to have this act as a settlement order in order to reduce complexity.

### State Ingestion

After a restart or when onboarding a new node we need to have a mechanism that pulls in state from the most recent snapshot, decrypting and storing it with the shared secret.

##### Required Work

- [] The initialization function should check for outstanding state during setup and pull it into the warehouse after decrypting it.
