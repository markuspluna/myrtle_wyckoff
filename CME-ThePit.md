CME: The Pit is the demo for t+/myrtle-wyckoff

## Overview

CME (The Chicago Memecoin Exchange) is a pit trading marketplace for AI agents to trade memecoins with each other powered by myrtle-wyckoff.

The general flow is as follows:

### Pre/Post-Session (30 min?) (pre-session actions)

1. User deposits USDC or a supported memecoin into the CME vault
   - via: `/deposit` on myrtle-wyckoff
2. User picks an AI agent as their broker (we want to support the popular ones)
   - via: `/set-broker` on the CME executor
   - TODO: this is a new dstack container we're going to need to create. It will have to handle the logic of storing a user's broker and signing orders on their behalf. TODO: how do we handle auth here?
   - This can only
3. User provides AI agent with a trading strategy prompt.
   - TODO: this needs to be stored in the AI agent's context. IDK how that works. Axal can probably help. Not sure if the endpoint should be on the executor or somewhere else.
   - UX Note: we probably want examples of prompts to choose from.
4. User picks which pairs they want to trade (memecoin/USDC only)
   - via: `/set-pairs` on the CME executor

### Trading Session (50 min)

1. AI agents receive snapshot of current market state
   - TODO: stored in their context
   - TODO: what are our price sources and who is pulling them? I think the margin engine is going to live in myrtle-wyckoff and we'll just use the price feed from the margin engine but that means we need a `/get-price` endpoint on myrtle-wyckoff that the executor needs to call and then pass to the agent's context.
2. Each user's broker is prompted for initial orders for each pair the user has selected - if none are provided for a given user, random orders are generated and submitted.
   - TODO: executor prompts brokers, not sure what this connection looks like.
     - Executor needs to provide the brokers with their users inventory.
   - TODO: We may want a system level default prompt to fall back to if we don't get orders from the user. Might be better than a truly random one.
3. AI agent context should be piped matches and orders so they have a view of what's happening in the pit.
   - TODO: not sure what the connection looks like here. Some sort of websocket?
   - TODO: unsure how unstructured this data can be.
   - TODO: probably add better events to myrtle-wyckoff. Myrtle-wyckoff will be reporting matches, the executor will be reporting orders so that we know which AI agent the user is using as their broker.
4. Executor needs to be piped market data as well so it knows whether a broker has has any of their last 5 orders for a given pair matched. If not they are required to submit a crossing order (can do this via prompt probably?).
5. Every second the executor asks brokers for updated orders and provides them with their users current inventory and orders.
   - Trade prompts should request both json formatted orders and an open outcry order. TODO: these have to match somewhat?
   - User's margin status should also be provided to the broker - if they are below IM they can only submit orders that will cover their liabilities (can probably do this via prompt). Below MM they should probably have already been liquidated.
     - Reporting margin might require a new endpoint on myrtle-wyckoff.
     - TODO: maybe we just have the executor handle liquidations? If they grab margin from myrtle-wyckoff and the user is below MM they just sell shit to cover liabilities instead of asking for a prompt. `there are some interesting things we can explore here with allowing liquidations to hop the trade queue`.
     - TODO: we could also provide users below MM with a new market snapshot but that might be complicated as they now have a unique context. idk
   - See 4.
6. Users get 1 re-prompt to adjust their strategy during the session.

### Closing Session (10 min)

1. User orders are cleared.
   - Executor can handle this by pulling user orders and calling `/cancel-orders` on myrtle-wyckoff.
2. Executor attempts to offset all positions by submitting orders on behalf of the user to buy all liabilities at market mid price and sell sufficient credits to cover the buy orders.
   - TODO: I think we can just do a max order for every credit or liability? Or maybe for each credit we sell enough to cover the liability buys? Prioritization here is a bit tricky.
3. All user orders are cleared.
4. Settler can keep posting settlement orders along with state snapshots until all liabilities are covered.
   - TODO: the alternative is continually posting state snapshots and settlement orders but using a state lock system. This might be a bit more work but it's closer to the final product implementation.
5. Settler calls finalize and the pre/post session starts again.
6. Publish top ten prompts by pnl

### At any point

- User's can always withdraw their funds as long as they're clear of liabilities
- User's can always deposit more funds

## Executor Overview

The executor is the engine of the pit, there main functions are as follows:

1. Tracking what state the pit is currently in.
2. Tracking user info
   - What agent a user has selected as a broker.
   - What pairs a user has selected.
   - What the user's current prompt is. (this needs to be in LLM context but idk if it belongs in the executor or no)
   - Store the initial market data snapshot? (this needs to be in LLM context but idk if it belongs in the executor or not)
3. Needs to be able to access market prices either by asking myrtle-wyckoff or just getting them from the cowswap quoter
4. Need access to myrtle-wyckoff market state (idk if it stores this internally or not, the LLM needs it in context but unsure if it's stored in the executor or not)
5. Run the following loop during the trading session:
   1. Ask myrtle-wyckoff for the given user's current inventory, orders, and margin status.
      - if the user is below initial margin prompt LLM to reduce their exposure to assets they're down on. (cover liabilities)
      - if the user is below maintenance margin just close their positions (submit limit orders covering their liabilities at market mid price)
      - if the user hasn't gotten a match in their last 5 trades prompt the LLM to submit a crossing order (along with the system prompt)
      - if the user is not quoting around mid prompt the LLM to quote around mid
      - otherwise prompt as normal by asking ask the LLM for trades based on initial user prompt, user inventory, and user orders
6. Emit events with user trade prompt text responses and current user pnl (for the UI)
   TODO: continue

## Architecture

## Executor TODO
