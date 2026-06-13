Cassie — Optimistic Oracle for Solana

Cassie is a permissionless optimistic oracle built for Solana. It enables anyone to ask questions and attach bounties, while answers are provided by bonded reporters. Disputes are resolved either through weighted community voting (using reputation) or escalated to a council when votes are too close.

Core concepts
- Question: A user posts a question and attaches a bounty.
- Answer: Any participant can submit an answer by placing a bond.
- Dispute: Answers can be disputed by staking bonds; disputed questions enter a dispute resolution flow.
- Reputation: Votes and proposals are weighted by a reputation score to reduce spam and reward reliable actors.
- Council: A set of trusted/community members who resolve close or contentious disputes. Council votes are also weighted by reputation.

High-level flow
1. Anyone asks a question and deposits a bounty.
2. Anyone can answer by placing the required bond.
3. Answers can be disputed by staking a bond against them.
4. If there are no disputes, the market resolves in favor of the party with the highest weighted support.
5. If results are close (no clear weighted majority), the question is escalated to the council.
6. Council members (who also stake bonds) vote; when quorum is reached the highest-weighted side wins and the question is settled.

On-chain instructions (brief)
- `initialize_config`: Create and initialize the global config (timers, fees, council members, treasury).
- `update_config`: Update selected config fields or freeze the config.
- `update_council`: Replace a council member (old -> new).
- `ask`: Post a question with a bounty and callback metadata.
- `propose`: Submit an answer/proposal for a question by staking a bond.
- `close_proposers`: Close proposer accounts after resolution.
- `dispute`: Dispute an answer by staking a bond and claiming an outcome.
- `council_vote`: Cast a council vote (bonded) during escalation.
- `finalize_council`: Finalize council voting and compute the result.
- `settle_question`: Settle the question, distribute rewards/slashes, and call the callback.
- `claim_reward`: Claim rewards or refunds for participants.
- `close_question`: Close and clean up question-related accounts.

Callback (external program)
Cassie's `settle_question` instruction calls an external callback program after settlement via a CPI (cross-program invocation). This lets any external program react to a settled question result.

Callback instruction data layout (8 + 32 + 1 bytes):
- bytes 0–7:   callback discriminator (u64LE, set at ask_question time)
- bytes 8–39:  question hash (32 bytes)
- byte 40:    result (0 = NO won, 1 = YES won)
- remaining:   any additional accounts forwarded from the settle call

Example callback program
A minimal callback program is provided under programs/callback_example/. It demonstrates how to receive and decode the callback from Cassie.

Key parts of the example:

```rust
// programs/callback_example/src/lib.rs

declare_id!("Ex4Y7eNFXJ5CE6JGCN8m4VY9vVnPPhXJHFP7ZQCqQ5zB");

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

Using the callback in Cassie:
When calling `ask`, provide:
- `callback_program`: the example program's ID (Ex4Y7eNFXJ5CE6JGCN8m4VY9vVnPPhXJHFP7ZQCqQ5zB)
- `callback_discriminator`: the 8-byte selector for `handle_cassie_result`
- `callback_data`: any account the callback program needs (passed as remaining accounts to `settle_question`)

The callback is only invoked if `callback_program` was set to a non-default key when the question was asked.

Features
- Permissionless question posting and answering with economic bonds
- Reputation-weighted voting to reward reliable participants
- Dispute resolution with escalation to an on-chain council
- Anchor/Rust Solana program with a Next.js landing app and runbooks for reproducible deployment

Repository structure
- programs/ — Anchor/Rust on-chain program(s)
- app/ — Next.js landing site and frontend code
- migrations/ — scripts for migrations / deployments
- runbooks/ — surfpool runbooks for deployment and localnet automation
- Anchor.toml, Cargo.toml — Anchor and Rust workspace configuration

Getting started (development)
Prerequisites
- Rust (recommended: use rustup and the toolchain defined in rust-toolchain.toml)
- Solana CLI (solana)
- Anchor (anchor-cli)
- Node.js and Yarn (for the frontend and workspace scripts)
- Surfpool (optional but recommended for localnet + runbooks)

Quick setup
1. Install Rust, Solana CLI, Anchor and Yarn. Example:
   - rustup: https://rustup.rs
   - Solana: https://docs.solana.com/cli/install-solana-cli-tools
   - Anchor: https://book.anchor-lang.com/installation.html
   - Yarn: https://classic.yarnpkg.com/lang/en/docs/install
2. (Optional) Install Surfpool for a local Surfnet: curl -sL https://run.surfpool.run/ | bash
3. Start a localnet (Surfpool) or ensure your provider is running:
   - surfpool start --watch

Build & test
- Build Rust programs: cargo build --workspace --release
- Run tests: cargo test
- Anchor tests (if present) will run with the configured provider; Anchor.toml has test = "cargo test"

Frontend
- Start the Next.js landing app:
  - yarn app:dev
  - or: cd app && yarn dev

Deployment
- This repository includes runbooks (runbooks/deployment) to automate deployment with Surfpool. Use Surfpool to run them:
  - surfpool run deployment
- There is also a TypeScript migration script in migrations/deploy.ts for program deployment and upgrades.

Contributing
- Open issues and PRs against this repository. Follow the code style and keep changes small and focused.
- If you change on-chain program interfaces, update the frontend/ABI and run migration scripts.

License
- This project is provided under the terms of the LICENSE file in this repository.

Contact
- For questions or help running the project locally, open an issue or contact the repository maintainers.

—
This README is intended to give developers and contributors a clear starting point. If you want I can add specific developer scripts, examples of posting a question via CLI, or a small integration test demonstrating the full ask/answer/dispute flow.
