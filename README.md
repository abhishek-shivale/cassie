# Cassie

**Cassie** is a permissionless optimistic oracle built for Solana. Anyone can post a question with a bounty, anyone can answer by bonding tokens, and disputes are resolved through reputation-weighted voting — escalating to a trusted council only when outcomes are genuinely contested.

[//]: # (> Built with [Anchor]&#40;https://book.anchor-lang.com/&#41; · Deployed on Devnet · Frontend in Next.js)

---

## Table of Contents

- [How It Works](#how-it-works)
- [Core Concepts](#core-concepts)
- [Resolution Flow](#resolution-flow)
- [On-Chain Instructions](#on-chain-instructions)
- [Callback System](#callback-system)
- [Repository Structure](#repository-structure)
- [Getting Started](#getting-started)
- [Deployment](#deployment)
- [Devnet Addresses](#devnet-addresses)
- [Contributing](#contributing)
- [License](#license)

---

## How It Works

1. **Ask** — Post a question and deposit a bounty.
2. **Propose** — Anyone stakes a bond and submits an answer.
3. **Dispute** — Answers can be challenged by staking a bond against them.
4. **Resolve** — If no disputes arise, the answer with the highest weighted support wins. If the result is too close, the question escalates to the council.
5. **Council** — Trusted members stake bonds and vote. When quorum is reached, the winning side is settled and rewards are distributed.
6. **Callback** — After settlement, an optional CPI fires into any registered external program.

---

## Core Concepts

| Concept | Description |
|---|---|
| **Question** | A user-posted query with an attached bounty and optional callback. |
| **Answer / Proposal** | A bonded response submitted by any participant. |
| **Dispute** | A challenge to an existing answer, also backed by a bond. |
| **Reputation** | An on-chain score that weights votes and proposals. Rewards reliable actors; penalizes bad ones. |
| **Council** | A set of trusted members who resolve close or contentious disputes. Council votes are reputation-weighted and bonded. |

---

## Resolution Flow

```
Ask (bounty deposited)
        │
        ▼
   Propose (bond)
        │
    dispute?
   ┌────┴────┐
  No         Yes
   │         │
   ▼         ▼
Settle    Dispute (bond)
(weighted      │
 majority)     │ 
               |
              YES        
               │          
               ▼          
            Council    
            Vote       
            (bonded,    
            quorum)
                │
                ▼
            Settle
                │
                ▼
            Callback CPI
            (if registered)
```

**Weighted majority** is determined by reputation-weighted support across all proposals. If the leading answer does not hold a clear supermajority, the question escalates to council.

---

## On-Chain Instructions

| Instruction | Description |
|---|---|
| `initialize_config` | Create and initialize global config — timers, fees, council members, treasury. |
| `update_config` | Update selected config fields or freeze the config. |
| `update_council` | Replace a council member (old → new). |
| `ask` | Post a question with a bounty and optional callback metadata. |
| `propose` | Submit an answer by staking a bond. |
| `close_proposers` | Close proposer accounts after resolution. |
| `dispute` | Dispute an existing answer by staking a bond and claiming an outcome. |
| `council_vote` | Cast a bonded council vote during escalation. |
| `finalize_council` | Finalize council voting and compute the result. |
| `settle_question` | Settle the question, distribute rewards/slashes, and fire the callback. |
| `claim_reward` | Claim rewards or refunds for individual participants. |
| `close_question` | Close and clean up all question-related accounts. |

---

## Callback System

After `settle_question` executes, Cassie fires a CPI into any external program registered at `ask` time. This lets downstream apps — prediction markets, insurance protocols, automated settlement vaults — react to a resolved outcome without polling.

### Callback instruction data layout

| Bytes | Field | Type |
|---|---|---|
| 0–7 | Callback discriminator | `u64` (LE) |
| 8–39 | Question hash | `[u8; 32]` |
| 40 | Result | `u8` (`0` = NO, `1` = YES) |
| 41+ | Additional accounts | Forwarded from `settle_question` remaining accounts |

### Minimal callback program

```rust
// programs/callback_example/src/lib.rs

declare_id!("");

#[program]
pub mod callback_example {
    use super::*;

    pub fn handle_cassie_result(
        ctx: Context<Callback>,
        hash: [u8; 32],
        result: bool,
    ) -> Result<()> {
        msg!(
            "Cassie callback — question hash: {:?}, result: {}",
            hash,
            result
        );
        msg!(
            "Result {} means the {} side won.",
            result,
            if result { "YES" } else { "NO" }
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Callback<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub callback_data: Account<'info, CallbackData>,
}

#[account]
pub struct CallbackData {
    pub data_len: u64,
}
```

### Registering a callback

When calling `ask`, provide:

- `callback_program` — the target program ID
- `callback_discriminator` — the 8-byte selector for the instruction to invoke
- `callback_data` — any accounts the callback program needs (passed as remaining accounts to `settle_question`)

The callback is **only invoked** if `callback_program` is set to a non-default key at ask time.

---

## Repository Structure

```
cassie/
├── programs/
│   ├── cassie/              # Core Anchor/Rust oracle program
│   └── callback_example/    # Minimal example callback program
├── app/                     # Next.js landing site and frontend
├── migrations/              # Deployment and upgrade scripts (TypeScript)
├── runbooks/                # Surfpool runbooks for reproducible deployments
├── Anchor.toml              # Anchor workspace config
└── Cargo.toml               # Rust workspace config
```

---

## Getting Started

### Prerequisites

| Tool | Install |
|---|---|
| Rust | [rustup.rs](https://rustup.rs) |
| Solana CLI | [docs.solana.com](https://docs.solana.com/cli/install-solana-cli-tools) |
| Anchor CLI | [book.anchor-lang.com](https://book.anchor-lang.com/installation.html) |
| Node.js + Yarn | [classic.yarnpkg.com](https://classic.yarnpkg.com/lang/en/docs/install) |
| Surfpool *(optional)* | `curl -sL https://run.surfpool.run/ \| bash` |

### Setup

```bash
# 1. Clone the repo
git clone https://github.com/your-org/cassie.git
cd cassie

# 2. Install JS dependencies
yarn install

# 3. Start a local network (Surfpool recommended)
surfpool start --watch
```

### Build

```bash
# Build all Rust programs
cargo build --workspace --release
```

### Test

```bash
# Run the full test suite
cargo test
```

### Frontend

```bash
# Start the Next.js app
yarn app:dev
# or
cd app && yarn dev
```

---

## Deployment

Cassie uses [Surfpool](https://surfpool.run) runbooks for reproducible deployments. There is also a TypeScript migration script for direct program deployment and upgrades.

```bash
# Deploy using Surfpool runbooks
surfpool run
surfpool run deployment --env localnet
```

---

## Devnet Addresses

| Program | Address |
|---|---|
| **Cassie** | `8XBYSkbwTEonoFRtqaU8PqbwyXaXvzDT1bApyUdRbrwf` |
| Cassie metadata | `9UdFewrrdmUKVuqo1K8BaUjcJq6kemTtsyLgaGyoT5fz` |
| **Callback Example** | `DANGHof54KqrvGnipP3Hm8whXXifmaWKQwYYH533jVaq` |
| Callback metadata | `2BWqpEZSAVsHLZZoyuT87F8b4nzhTLPJTDDLndN45r7w` |

---

## Contributing

- Open issues and PRs against this repository.
- Keep changes small and focused; one concern per PR.
- If you modify any on-chain program interfaces, update the corresponding frontend/ABI and run migration scripts before opening a PR.

---

## License

This project is provided under the terms of the [LICENSE](./LICENSE) file in this repository.