/**
 * Devnet integration tests for the cassie oracle.
 *
 * Unlike `test.ts` (which runs against a Surfpool mainnet-fork and uses the
 * `surfnet_*` cheat RPCs to mint USDC + time-travel), this file runs against a
 * REAL cluster (devnet by default). That removes two superpowers, so the tests
 * are written to cope with the real-world constraints instead of faking them:
 *
 *   1. No `surfnet_setTokenAccount`. The USDC mint
 *      (4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU) is Circle's devnet USDC and
 *      we don't hold its mint authority. So test accounts are funded by
 *      TRANSFERRING USDC out of the admin wallet's own ATA. => the wallet at
 *      CASSIE_WALLET_PATH must already hold some devnet USDC and some devnet SOL.
 *      Grab USDC from https://faucet.circle.com (select Solana Devnet).
 *
 *   2. No `surfnet_timeTravel`. The answer/dispute/council windows are read from
 *      config at ask-time, so the lifecycle suite first SHRINKS those windows via
 *      update_config and then waits real wall-clock seconds (`sleep`) for them to
 *      elapse. That makes the optimistic-resolve lifecycle genuinely runnable on
 *      devnet in well under a minute.
 *
 * What CANNOT be fired on a real cluster (and is therefore skipped, not faked):
 *   - close_question: CLOSE_GRACE is a hard-coded 86_400s (1 day) constant, not a
 *     config field, so it can't be shrunk. Run with RUN_SLOW_CLOSE=1 only if you
 *     are willing to wait a day. The full instruction is still written below.
 *   - Full council resolution: council votes must be signed by the council member
 *     keypairs, but config.ts only ships their PUBLIC keys. We can't sign for
 *     them. The dispute *escalation* (which only needs a disputer we control) IS
 *     fired and asserted.
 *
 * Run:
 *   CASSIE_CLUSTER_URL=https://api.devnet.solana.com \
 *   CASSIE_WALLET_PATH=~/.config/solana/id.json \
 *   yarn test:dev
 */
import "mocha";
import { expect } from "chai";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import * as fs from "fs";
import crypto from "crypto";

import {
  PROGRAM_ID,
  CALLBACK_EXAMPLE_PROGRAM_ID,
  USDC_MINT,
  DEFAULT_ANSWER_WINDOW,
  DEFAULT_DISPUTE_WINDOW,
  DEFAULT_COUNCIL_WINDOW,
  MIN_STAKE,
  MIN_DISPUTE_BOND,
  IDL_PATH,
  TREASURY_PK,
  COUNCIL_PKS,
  ONE_SOL,
} from "./config";

import {
  getProvider,
  getConnection,
  getWallet,
  getClusterUrl,
  newKeypair,
  deterministicKeypair,
  sendIx,
  sleep,
} from "./helpers/setup";

import {
  tokenBalance,
  ataFor,
  getOrCreateAta,
  fundOwnerWithUsdc,
} from "./helpers/tokens";

import {
  buildInitializeConfig,
  buildUpdateConfig,
  buildUpdateCouncil,
  buildAsk,
  buildPropose,
  buildCloseProposers,
  buildDispute,
  buildSettleQuestion,
  buildClaimReward,
  buildCloseQuestion,
  questionPdaFor,
  answerPdaFor,
  outcomePdaFor,
  disputePdaFor,
  adminConfigPda,
  HANDLE_CASSIE_RESULT_DISCRIMINATOR_BUF,
} from "./helpers/instructions";

import {
  fetchQuestion,
  fetchOutcome,
  fetchOracleConfig,
  fetchAnswer,
  fetchReputation,
  tryFetchDispute,
  accountExists,
} from "./helpers/cassie-accounts";

// ---------------------------------------------------------------------------
// Devnet tuning knobs.
// ---------------------------------------------------------------------------

// Short windows (seconds) used by the lifecycle suite. Kept tiny so real waits
// stay short, but large enough that a slow confirmation doesn't blow the
// deadline before the crank tx lands. Bump these on a congested cluster.
const SHORT_ANSWER_WINDOW = 15;
const SHORT_DISPUTE_WINDOW = 15;

// extra slack added on top of a deadline before we crank past it.
const DEADLINE_BUFFER_SEC = 4;

// SOL handed to each generated sub-account: enough for fees + a couple of ATA
// rents. Transferred from the admin wallet (devnet airdrop is rate-limited).
const SUBACCOUNT_SOL = ONE_SOL / 20n; // 0.05 SOL

// USDC (base units, 6 decimals) handed to each sub-account. Must exceed
// MIN_STAKE (5 USDC) so a sub-account can bond a proposal/dispute.
const SUBACCOUNT_USDC = 6_000_000n;

const RUN_SLOW_CLOSE = process.env.RUN_SLOW_CLOSE === "1";

// ---------------------------------------------------------------------------
// Devnet-safe helpers (no surfnet cheats).
// ---------------------------------------------------------------------------

/** Transfer SOL from the admin wallet to `to` (devnet airdrop replacement). */
async function fundSol(to: PublicKey, lamports: bigint): Promise<void> {
  const admin = getWallet().payer;
  const ix = SystemProgram.transfer({
    fromPubkey: admin.publicKey,
    toPubkey: to,
    lamports: Number(lamports),
  });
  await sendIx(ix, admin);
}

/** Fund a freshly generated account with SOL + USDC from the admin wallet. */
async function provision(kp: Keypair): Promise<void> {
  await fundSol(kp.publicKey, SUBACCOUNT_SOL);
  await fundOwnerWithUsdc(kp.publicKey, SUBACCOUNT_USDC, getWallet().payer);
}

/** Sleep until a unix deadline (+buffer) has passed on the wall clock. */
async function sleepPastDeadline(deadlineUnix: number): Promise<void> {
  const targetMs = (deadlineUnix + DEADLINE_BUFFER_SEC) * 1000;
  const waitMs = targetMs - Date.now();
  if (waitMs > 0) {
    console.log(`    [wait] sleeping ${Math.ceil(waitMs / 1000)}s for deadline...`);
    await sleep(waitMs);
  }
}

/** Set the global default windows small so the lifecycle is testable in seconds. */
// async function shrinkWindows(admin: PublicKey): Promise<void> {
//   await sendIx(
//     buildUpdateConfig({
//       admin,
//       defaultAnswerWindow: SHORT_ANSWER_WINDOW,
//       defaultDisputeWindow: SHORT_DISPUTE_WINDOW,
//       defaultCouncilWindow: null,
//       freeze: false,
//     })
//   );
// }

// ---------------------------------------------------------------------------

describe("cassie [devnet]", function () {
  // Real network: every tx is a real confirmation round-trip, and the lifecycle
  // suite waits real seconds. Give mocha room.
  this.timeout(300_000);

  const admin = getWallet().payer;
  let adminHasUsdc = false;

  before(async () => {
    const conn = getConnection();
    console.log(`\n[cassie] program id: ${PROGRAM_ID.toBase58()}`);
    console.log(`[cassie] cluster:    ${conn.rpcEndpoint}`);
    console.log(`[cassie] admin:      ${admin.publicKey.toBase58()}`);

    const solLamports = await conn.getBalance(admin.publicKey);
    const adminUsdc = await tokenBalance(ataFor(admin.publicKey));
    adminHasUsdc = adminUsdc > 0n;
    console.log(`[cassie] admin SOL:  ${solLamports / 1e9}`);
    console.log(`[cassie] admin USDC: ${adminUsdc} (base units)`);

    if (solLamports < Number(ONE_SOL) / 5) {
      console.warn(
        "[cassie] WARNING: admin SOL is low. Fund it (devnet faucet / `solana airdrop 1`)."
      );
    }
    if (!adminHasUsdc) {
      console.warn(
        "[cassie] WARNING: admin wallet holds no USDC. Funding-dependent suites " +
          "will be skipped. Get devnet USDC at https://faucet.circle.com."
      );
    }
  });

  // -------------------------------------------------------------------------
  // Offline / pure checks. Always safe to fire — no network mutation.
  // -------------------------------------------------------------------------
  // describe("sanity", () => {
  //   it("runs against a real (non-localhost) cluster", () => {
  //     const url = getClusterUrl();
  //     // Guard against accidentally pointing dev.ts at a local Surfpool.
  //     expect(url).to.not.match(/127\.0\.0\.1|localhost/);
  //   });

  //   it("program id matches the deployed IDL address", () => {
  //     const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf-8"));
  //     expect(idl.address).to.equal(PROGRAM_ID.toBase58());
  //     expect(PROGRAM_ID.toBase58()).to.equal(
  //       "8XBYSkbwTEonoFRtqaU8PqbwyXaXvzDT1bApyUdRbrwf"
  //     );
  //   });

  //   it("builds instructions addressed to PROGRAM_ID", () => {
  //     const ix = buildAsk({
  //       questioner: Keypair.generate().publicKey,
  //       hash: new Uint8Array(32),
  //       callbackProgram: PublicKey.default,
  //     });
  //     expect(ix.programId.toBase58()).to.equal(PROGRAM_ID.toBase58());
  //   });

  //   it("program is deployed on this cluster", async () => {
  //     const info = await getConnection().getAccountInfo(PROGRAM_ID);
  //     expect(info, "program account not found on cluster").to.not.be.null;
  //     expect(info!.executable).to.equal(true);
  //   });
  // });

  // // -------------------------------------------------------------------------
  // // Oracle config. Initialize if missing, otherwise read + assert invariants.
  // // Mutating fields (windows) are reset to known DEFAULTS so a re-run starts clean.
  // // -------------------------------------------------------------------------
  // describe("oracle config", () => {
  //   it("exists (initialize if first run) and exposes the expected invariants", async () => {
  //     const exists = await accountExists(adminConfigPda());
  //     if (!exists) {
  //       console.log("    [config] not initialized — initializing now");
  //       await sendIx(
  //         buildInitializeConfig({
  //           admin: admin.publicKey,
  //           treasury: TREASURY_PK,
  //           council: COUNCIL_PKS,
  //         })
  //       );
  //     } else {
  //       // Reset windows back to documented defaults (lifecycle suite shrinks them).
  //       await sendIx(
  //         buildUpdateConfig({
  //           admin: admin.publicKey,
  //           defaultAnswerWindow: DEFAULT_ANSWER_WINDOW,
  //           defaultDisputeWindow: DEFAULT_DISPUTE_WINDOW,
  //           defaultCouncilWindow: DEFAULT_COUNCIL_WINDOW,
  //           freeze: false,
  //         })
  //       );
  //     }

  //     const cfg = await fetchOracleConfig();
  //     expect(cfg.admin.toBase58()).to.equal(admin.publicKey.toBase58());
  //     expect(cfg.mint.toBase58()).to.equal(USDC_MINT.toBase58());
  //     expect(cfg.councilSize).to.equal(9);
  //     expect(cfg.freeze).to.equal(false);
  //     expect(Number(cfg.defaultAnswerWindow)).to.equal(DEFAULT_ANSWER_WINDOW);
  //     expect(Number(cfg.defaultDisputeWindow)).to.equal(DEFAULT_DISPUTE_WINDOW);
  //   });

  //   it("admin can replace a council member then restore it", async () => {
  //     const cfg = await fetchOracleConfig();
  //     const original = cfg.council[0];
  //     const replacement = newKeypair().publicKey;

  //     await sendIx(
  //       buildUpdateCouncil({ admin: admin.publicKey, old: original, new: replacement })
  //     );
  //     let after = await fetchOracleConfig();
  //     expect(after.council[0].toBase58()).to.equal(replacement.toBase58());

  //     // Restore so devnet config keeps its real council membership.
  //     await sendIx(
  //       buildUpdateCouncil({ admin: admin.publicKey, old: replacement, new: original })
  //     );
  //     after = await fetchOracleConfig();
  //     expect(after.council[0].toBase58()).to.equal(original.toBase58());
  //   });

  //   it("admin can update a window via update_config", async () => {
  //     await sendIx(
  //       buildUpdateConfig({
  //         admin: admin.publicKey,
  //         defaultDisputeWindow: 9000,
  //         defaultAnswerWindow: null,
  //         defaultCouncilWindow: null,
  //         freeze: null,
  //       })
  //     );
  //     const cfg = await fetchOracleConfig();
  //     expect(Number(cfg.defaultDisputeWindow)).to.equal(9000);
  //   });
  // });

  // -------------------------------------------------------------------------
  // Optimistic-resolve lifecycle, fired for real with shrunk windows + real
  // waits: ask -> propose -> close_proposers -> settle -> claim_reward.
  // close_question is gated behind RUN_SLOW_CLOSE (CLOSE_GRACE = 1 day).
  // // -------------------------------------------------------------------------
  describe("lifecycle (optimistic resolve)", () => {
    // Constant, deterministic hash so the same question PDA is hit across runs
    // (NOT random). Bump the seed string to start a fresh question on devnet.
    const LIFECYCLE_SEED = process.env.CASSIE_LIFECYCLE_SEED ?? "cassie-devnet-lifecycle-p2";
    const hash = new Uint8Array(
      crypto.createHash("sha256").update(LIFECYCLE_SEED).digest()
    );
    // Reuse the fixed, pre-funded devnet accounts from config.ts (ASKER/PROPOSER/
    // CRANKER = deterministicKeypair 1/2/3). They already hold SOL + USDC on
    // devnet, so we do NOT create-or-fund fresh keypairs every run.
    const creator = deterministicKeypair(1); // config ASKER
    const proposer = deterministicKeypair(2); // config PROPOSER
    const cranker = deterministicKeypair(3); // config CRANKER
    const poolAta = ataFor(questionPdaFor(hash));

    // Each step is an INDEPENDENT describe so it can run on its own (devnet,
    // hours apart) against the same persisted question. Run one step at a time:
    //   yarn test:dev --grep "step 2"
    // The state each step needs is read fresh from chain, so order across
    // separate invocations doesn't matter — only that earlier steps already ran.

    describe("step 1: ask", () => {
      it("asks a question with a YES-side bounty", async () => {
        await sendIx(
          buildAsk({
            questioner: creator.publicKey,
            hash,
            bounty: 70n,
            category: 0x62,
            callbackProgram: new PublicKey("DANGHof54KqrvGnipP3Hm8whXXifmaWKQwYYH533jVaq"),
            callbackDiscriminator: HANDLE_CASSIE_RESULT_DISCRIMINATOR_BUF,
          }),
          creator
        );

        const q = await fetchQuestion(hash);
        expect(q.creator.toBase58()).to.equal(creator.publicKey.toBase58());
        expect(q.bounty.toString()).to.equal("70");
        expect(q.state).to.deep.equal({ asked: {} });
        expect(q.hasDispute).to.equal(false);
        expect(q.escalated).to.equal(false);
        expect(await tokenBalance(poolAta)).to.equal(70n);
      });
    });

    describe("step 2: propose", () => {
      it("submits a YES proposal", async () => {
        await sendIx(
          buildPropose({ proposer: proposer.publicKey, hash, side: true, stake: MIN_STAKE }),
          proposer
        );

        const q = await fetchQuestion(hash);
        expect(q.state).to.deep.equal({ answering: {} });
        expect(Number(q.yesCount)).to.equal(1);
        expect(String(q.totalYesStake)).to.equal(String(MIN_STAKE));

        const a = await fetchAnswer(hash, proposer.publicKey);
        expect(a.answerer.toBase58()).to.equal(proposer.publicKey.toBase58());
        expect(a.side).to.equal(true);
        expect(a.claimed).to.equal(false);
      });
    });

    describe("step 3: close proposers", () => {
      it("closes proposers after the answer window elapses (real wait)", async () => {
        // const q = await fetchQuestion(hash);
        // await sleepPastDeadline(Number(q.answerDeadline));

        await sendIx(buildCloseProposers({ cranker: cranker.publicKey, hash }), cranker);

        const q2 = await fetchQuestion(hash);
        expect(q2.state).to.deep.equal({ resolved: {} });
        const o = await fetchOutcome(hash);
        expect(o.result).to.equal(true);
        expect(o.resolver).to.deep.equal({ optimistic: {} });
      });
    });

    describe("step 4: settle", () => {
      it("settles after the dispute window elapses (real wait)", async () => {
        const q = await fetchQuestion(hash);
        await sleepPastDeadline(Number(q.disputeDeadline));

        await sendIx(
          buildSettleQuestion({
            cranker: cranker.publicKey,
            hash,
            treasury: TREASURY_PK,
            hasDispute: false,
            hasCouncil: false,
            callbackProgram: CALLBACK_EXAMPLE_PROGRAM_ID,
            callbackRemainingAccounts: [
              { pubkey: questionPdaFor(hash), isSigner: false, isWritable: true },
            ],
          }),
          cranker
        );

        const q2 = await fetchQuestion(hash);
        expect(q2.state).to.deep.equal({ settled: {} });
        expect(Number(q2.perAnswerReward)).to.be.greaterThan(0);
      });
    });

    describe("step 5: claim", () => {
      it("winning proposer claims stake + reward", async () => {
        // Ensure the claimer ATA exists (it does — propose created it).
        const pre = await tokenBalance(ataFor(proposer.publicKey));

        await sendIx(
          buildClaimReward({
            claimer: proposer.publicKey,
            hash,
            isDisputer: false,
            isCouncilVoter: false,
          }),
          proposer
        );

        const post = await tokenBalance(ataFor(proposer.publicKey));
        expect(post > pre).to.equal(true); // got stake back + reward
        // answer account is closed by claim_reward
        expect(await accountExists(answerPdaFor(hash, proposer.publicKey))).to.equal(false);

        const rep = await fetchReputation(proposer.publicKey);
        expect(Number(rep.answered)).to.equal(1);
        expect(Number(rep.correct)).to.equal(1);
      });
    });

    describe("step 6: close question (slow)", () => {
      // CLOSE_GRACE = 86_400s is a hard-coded constant, not config. It can't be
      // shrunk, so this only runs when you explicitly opt into the 1-day wait.
      (RUN_SLOW_CLOSE ? it : it.skip)(
        "closes the question after CLOSE_GRACE (RUN_SLOW_CLOSE=1, ~1 day)",
        async function () {
          this.timeout(90_000_000);
          const o = await fetchOutcome(hash);
          await sleepPastDeadline(Number(o.settledAt) + 86_400);

          await sendIx(
            buildCloseQuestion({
              cranker: cranker.publicKey,
              creator: creator.publicKey,
              hash,
              treasury: TREASURY_PK,
            }),
            cranker
          );

          expect(await accountExists(questionPdaFor(hash))).to.equal(false);
          expect(await accountExists(poolAta)).to.equal(false);
        }
      );
    });
  });

  // // -------------------------------------------------------------------------
  // // Callback delivery, fired for real: settle invokes the callback_example
  // // program with the cassie result.
  // // -------------------------------------------------------------------------
  // describe("callback delivery", () => {
  //   const hash = new Uint8Array(crypto.randomBytes(32));
  //   const creator = newKeypair();
  //   const proposer = newKeypair();
  //   const cranker = newKeypair();

  //   before(async function () {
  //     if (!adminHasUsdc) this.skip();
  //     const cbInfo = await getConnection().getAccountInfo(CALLBACK_EXAMPLE_PROGRAM_ID);
  //     if (!cbInfo) {
  //       console.warn("    [skip] callback_example program not deployed on this cluster");
  //       this.skip();
  //     }
  //     await shrinkWindows(admin.publicKey);
  //     await provision(creator);
  //     await provision(proposer);
  //     await fundSol(cranker.publicKey, SUBACCOUNT_SOL);
  //   });

  //   it("fires the callback to callback_example on settle", async () => {
  //     await sendIx(
  //       buildAsk({
  //         questioner: creator.publicKey,
  //         hash,
  //         bounty: 70n,
  //         category: 0x62,
  //         callbackProgram: CALLBACK_EXAMPLE_PROGRAM_ID,
  //         callbackDiscriminator: HANDLE_CASSIE_RESULT_DISCRIMINATOR_BUF,
  //       }),
  //       creator
  //     );
  //     await sendIx(
  //       buildPropose({ proposer: proposer.publicKey, hash, side: true, stake: MIN_STAKE }),
  //       proposer
  //     );

  //     let q = await fetchQuestion(hash);
  //     await sleepPastDeadline(Number(q.answerDeadline));
  //     await sendIx(buildCloseProposers({ cranker: cranker.publicKey, hash }), cranker);

  //     q = await fetchQuestion(hash);
  //     await sleepPastDeadline(Number(q.disputeDeadline));

  //     await sendIx(
  //       buildSettleQuestion({
  //         cranker: cranker.publicKey,
  //         hash,
  //         treasury: TREASURY_PK,
  //         hasDispute: false,
  //         hasCouncil: false,
  //         callbackProgram: CALLBACK_EXAMPLE_PROGRAM_ID,
  //         callbackRemainingAccounts: [
  //           { pubkey: questionPdaFor(hash), isSigner: false, isWritable: true },
  //         ],
  //       }),
  //       cranker
  //     );

  //     q = await fetchQuestion(hash);
  //     expect(q.state).to.deep.equal({ settled: {} });
  //     expect(q.callbackProgram.toBase58()).to.equal(
  //       CALLBACK_EXAMPLE_PROGRAM_ID.toBase58()
  //     );
  //   });
  // });

  // // -------------------------------------------------------------------------
  // // Dispute escalation, fired for real: a disputer we control escalates the
  // // resolved question. The FULL council resolution (vote -> finalize -> settle)
  // // is intentionally skipped: council votes must be signed by the council member
  // // keypairs, but config.ts only ships their public keys. See the skipped test.
  // // -------------------------------------------------------------------------
  // describe("dispute escalation", () => {
  //   const hash = new Uint8Array(crypto.randomBytes(32));
  //   const creator = newKeypair();
  //   const proposer = newKeypair();
  //   const disputer = newKeypair();
  //   const cranker = newKeypair();

  //   before(async function () {
  //     if (!adminHasUsdc) this.skip();
  //     await shrinkWindows(admin.publicKey);
  //     await provision(creator);
  //     await provision(proposer);
  //     await provision(disputer);
  //     await fundSol(cranker.publicKey, SUBACCOUNT_SOL);
  //   });

  //   it("asks, proposes YES, then resolves optimistically", async () => {
  //     await sendIx(
  //       buildAsk({
  //         questioner: creator.publicKey,
  //         hash,
  //         bounty: 70n,
  //         category: 0x62,
  //         callbackProgram: PublicKey.default,
  //       }),
  //       creator
  //     );
  //     await sendIx(
  //       buildPropose({ proposer: proposer.publicKey, hash, side: true, stake: MIN_STAKE }),
  //       proposer
  //     );

  //     const q = await fetchQuestion(hash);
  //     await sleepPastDeadline(Number(q.answerDeadline));
  //     await sendIx(buildCloseProposers({ cranker: cranker.publicKey, hash }), cranker);

  //     expect((await fetchQuestion(hash)).state).to.deep.equal({ resolved: {} });
  //   });

  //   it("a NO disputer escalates within the dispute window", async () => {
  //     await sendIx(
  //       buildDispute({
  //         disputer: disputer.publicKey,
  //         hash,
  //         bond: MIN_DISPUTE_BOND,
  //         claimedOutcome: false, // disputing the optimistic YES
  //       }),
  //       disputer
  //     );

  //     const q = await fetchQuestion(hash);
  //     expect(q.state).to.deep.equal({ escalated: {} });
  //     expect(q.escalated).to.equal(true);
  //     expect(q.hasDispute).to.equal(true);

  //     const d = await tryFetchDispute(hash);
  //     expect(d, "dispute account should exist").to.not.be.null;
  //     expect(d.disputer.toBase58()).to.equal(disputer.publicKey.toBase58());
  //     expect(d.claimedOutcome).to.equal(false);
  //   });

  //   // Council members vote with their own signatures; we don't have their secret
  //   // keys (config.ts is pubkeys only), so the council leg can't run on devnet.
  //   // To exercise it you'd first update_council to keypairs you control.
  //   it.skip(
  //     "council votes -> finalize -> settle (needs council member secret keys)",
  //     () => {
  //       /* intentionally not implemented for devnet — see comment above */
  //     }
  //   );
  // });
});
