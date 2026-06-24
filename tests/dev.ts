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

const SHORT_ANSWER_WINDOW = 15;
const SHORT_DISPUTE_WINDOW = 15;

const DEADLINE_BUFFER_SEC = 4;

const SUBACCOUNT_SOL = ONE_SOL / 20n;

const SUBACCOUNT_USDC = 6_000_000n;

const RUN_SLOW_CLOSE = process.env.RUN_SLOW_CLOSE === "1";

async function fundSol(to: PublicKey, lamports: bigint): Promise<void> {
  const admin = getWallet().payer;
  const ix = SystemProgram.transfer({
    fromPubkey: admin.publicKey,
    toPubkey: to,
    lamports: Number(lamports),
  });
  await sendIx(ix, admin);
}

async function provision(kp: Keypair): Promise<void> {
  await fundSol(kp.publicKey, SUBACCOUNT_SOL);
  await fundOwnerWithUsdc(kp.publicKey, SUBACCOUNT_USDC, getWallet().payer);
}

async function sleepPastDeadline(deadlineUnix: number): Promise<void> {
  const targetMs = (deadlineUnix + DEADLINE_BUFFER_SEC) * 1000;
  const waitMs = targetMs - Date.now();
  if (waitMs > 0) {
    console.log(`    [wait] sleeping ${Math.ceil(waitMs / 1000)}s for deadline...`);
    await sleep(waitMs);
  }
}

describe("cassie [devnet]", function () {
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

  describe("lifecycle (optimistic resolve)", () => {
    const LIFECYCLE_SEED = process.env.CASSIE_LIFECYCLE_SEED ?? "cassie-devnet-lifecycle-p2";
    const hash = new Uint8Array(
      crypto.createHash("sha256").update(LIFECYCLE_SEED).digest()
    );
    const creator = deterministicKeypair(1);
    const proposer = deterministicKeypair(2);
    const cranker = deterministicKeypair(3);
    const poolAta = ataFor(questionPdaFor(hash));

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
        expect(post > pre).to.equal(true);
        expect(await accountExists(answerPdaFor(hash, proposer.publicKey))).to.equal(false);

        const rep = await fetchReputation(proposer.publicKey);
        expect(Number(rep.answered)).to.equal(1);
        expect(Number(rep.correct)).to.equal(1);
      });
    });

    describe("step 6: close question (slow)", () => {
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
});
