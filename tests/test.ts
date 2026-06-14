import "mocha";
import { expect } from "chai";
import { Keypair, PublicKey } from "@solana/web3.js";
import * as fs from "fs";
import crypto from "crypto";

import {
  PROGRAM_ID,
  CALLBACK_EXAMPLE_PROGRAM_ID,
  USDC_MINT,
  DEFAULT_ANSWER_WINDOW,
  DEFAULT_DISPUTE_WINDOW,
  DEFAULT_COUNCIL_WINDOW,
  SLASH_BPS,
  TREASURY_BPS,
  COUNCIL_BPS,
  DIVERGENCE_BPS,
  MIN_STAKE,
  IDL_PATH,
  MIN_BOUNTY,
  ASKER,
  PROPOSER,
  CRANKER,
  TREASURY_PK,
  COUNCIL_PKS,
} from "./config";

import {
  getProvider,
  getProgram,
  getConnection,
  getWallet,
  newKeypair,
  airdrop,
  sendIx,
  sleep,
  deterministicKeypair,
  timeTravel,
} from "./helpers/setup";

import {
  tokenBalance,
  ataFor,
  surfnetFundUsdc,
} from "./helpers/tokens";

import {
  buildInitializeConfig,
  buildUpdateConfig,
  buildUpdateCouncil,
  buildAsk,
  buildPropose,
  buildCloseProposers,
  buildSettleQuestion,
  buildClaimReward,
  buildCloseQuestion,
  questionPdaFor,
  answerPdaFor,
  adminConfigPda,
  HANDLE_CASSIE_RESULT_DISCRIMINATOR_BUF,
} from "./helpers/instructions";

import {
  fetchQuestion,
  fetchOutcome,
  fetchOracleConfig,
  fetchAnswer,
  fetchReputation,
  accountExists,
} from "./helpers/cassie-accounts";

describe("cassie", () => {
  const admin = getWallet().payer;
  const treasury = deterministicKeypair(4);

  before(async () => {
    console.log(`\n[cassie] program id: ${PROGRAM_ID.toBase58()}`);
    console.log(`[cassie] cluster:    ${getConnection().rpcEndpoint}`);
    console.log(`[cassie] admin:      ${admin.publicKey.toBase58()}`);
  });

  it("program id is interchangeable", () => {
    expect(PROGRAM_ID.toBase58()).to.equal("8XBYSkbwTEonoFRtqaU8PqbwyXaXvzDT1bApyUdRbrwf");
    const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf-8"));
    expect(idl.address).to.equal(PROGRAM_ID.toBase58());
  });

  it("builds instructions addressed to the configured PROGRAM_ID", () => {
    const ix = buildAsk({
      questioner: Keypair.generate().publicKey,
      hash: new Uint8Array(32),
      callbackProgram: PublicKey.default,
    });
    expect(ix.programId.toBase58()).to.equal(PROGRAM_ID.toBase58());
  });

  describe("initialize_config", () => {
    it("initializes the oracle config", async () => {
      // Surfpool is persistent — the PDA may already exist from a previous run.
      // If so, reset the mutable fields back to defaults instead of re-initialising.
      await surfnetFundUsdc(admin.publicKey, 10000n)
      await surfnetFundUsdc(treasury.publicKey, 10000n)
      const configExists = await accountExists(adminConfigPda());
      if (configExists) {
        await sendIx(
          buildUpdateConfig({
            admin: admin.publicKey,
            defaultAnswerWindow: DEFAULT_ANSWER_WINDOW,
            defaultDisputeWindow: DEFAULT_DISPUTE_WINDOW,
            defaultCouncilWindow: DEFAULT_COUNCIL_WINDOW,
            freeze: false,
          })
        );
      } else {
        await sendIx(
          buildInitializeConfig({
            admin: admin.publicKey,
            treasury: TREASURY_PK,
            council: COUNCIL_PKS,
          })
        );
      }

      const cfg = await fetchOracleConfig();
      expect(cfg.admin.toBase58()).to.equal(admin.publicKey.toBase58());
      expect(cfg.treasury.toBase58()).to.equal(TREASURY_PK.toBase58());
      expect(cfg.mint.toBase58()).to.equal(USDC_MINT.toBase58());
      expect(cfg.councilSize).to.equal(9);
      expect(cfg.quorum).to.equal(6);
      expect(Number(cfg.defaultAnswerWindow)).to.equal(DEFAULT_ANSWER_WINDOW);
      expect(Number(cfg.defaultDisputeWindow)).to.equal(DEFAULT_DISPUTE_WINDOW);
      expect(Number(cfg.defaultCouncilWindow)).to.equal(DEFAULT_COUNCIL_WINDOW);
      expect(String(cfg.minBounty)).to.equal(String(MIN_BOUNTY));
      expect(String(cfg.minStake)).to.equal(String(MIN_STAKE));
      expect(String(cfg.minDisputeBond)).to.equal(String(MIN_STAKE));
      expect(Number(cfg.slashBps)).to.equal(SLASH_BPS);
      expect(Number(cfg.treasuryBps)).to.equal(TREASURY_BPS);
      expect(Number(cfg.councilBps)).to.equal(COUNCIL_BPS);
      expect(Number(cfg.divergenceBps)).to.equal(DIVERGENCE_BPS);
      expect(cfg.freeze).to.equal(false);
    });
  });

  describe("update_council + update_config", () => {
    it("replaces a council member", async () => {
      const newMember = newKeypair();
      const cfg = await fetchOracleConfig();
      const oldMember = cfg.council[0];
      const ix = buildUpdateCouncil({
        admin: admin.publicKey,
        old: oldMember,
        new: newMember.publicKey,
      });
      await sendIx(ix);
  
      const cfg2 = await fetchOracleConfig();
      expect(cfg2.council[0].toBase58()).to.equal(newMember.publicKey.toBase58());
      expect(cfg2.council[1].toBase58()).to.equal(cfg.council[1].toBase58());
    });
  
    it("updates default_dispute_window", async () => {
      const ix = buildUpdateConfig({
        admin: admin.publicKey,
        defaultDisputeWindow: 7500,
        defaultAnswerWindow: null,
        defaultCouncilWindow: null,
        freeze: null,
      });
      await sendIx(ix);
  
      const cfg = await fetchOracleConfig();
      expect(Number(cfg.defaultDisputeWindow)).to.equal(7500);
    });
  });

  describe("happy path (ask -> propose -> close -> settle -> claim -> close_question)", () => {
    const hash = new Uint8Array(crypto.randomBytes(32));
    const creator = newKeypair();
    const proposer = newKeypair();
    const cranker = newKeypair();
    let poolAta: PublicKey;
  
    before(async () => {
      await airdrop(creator.publicKey, 1n);
      await airdrop(proposer.publicKey, 1n);
      await airdrop(cranker.publicKey, 1n);
      // Surfpool is a mainnet fork — USDC mint already exists, no mint authority needed.
      // Directly write funded token accounts via surfnet_setAccount.
      await surfnetFundUsdc(creator.publicKey, 100_000n);
      await surfnetFundUsdc(proposer.publicKey, 100_000n);
      poolAta = ataFor(questionPdaFor(hash));
    });
  
    it("asks a question with a YES-side bounty", async () => {
      const ix = buildAsk({
        questioner: creator.publicKey,
        hash,
        bounty: 70n,
        category: 0x62,
        callbackProgram: PublicKey.default,
      });
      await sendIx(ix, creator);
  
      const q = await fetchQuestion(hash);
      expect(q.creator.toBase58()).to.equal(creator.publicKey.toBase58());
      expect(q.bounty.toString()).to.equal("70");
      expect(q.state).to.deep.equal({ asked: {} });
      expect(Number(q.yesCount)).to.equal(0);
      expect(Number(q.noCount)).to.equal(0);
      expect(q.hasDispute).to.equal(false);
      expect(q.escalated).to.equal(false);
      expect(Number(q.answerDeadline)).to.equal(
        Number(q.createdAt) + DEFAULT_ANSWER_WINDOW
      );
      expect(Number(q.perAnswerReward)).to.equal(0);
      expect(Number(q.councilRewardPerVote)).to.equal(0);
  
      const bountyAtaBalance = await tokenBalance(poolAta);
      expect(bountyAtaBalance).to.equal(70n);
    });
  
    it("submits a YES proposal", async () => {
      const ix = buildPropose({
        proposer: proposer.publicKey,
        hash,
        side: true,
        stake: MIN_STAKE,
      });
      await sendIx(ix, proposer);
  
      const q = await fetchQuestion(hash);
      expect(q.state).to.deep.equal({ answering: {} });
      expect(Number(q.yesCount)).to.equal(1);
      expect(Number(q.noCount)).to.equal(0);
      expect(String(q.totalYesWeight)).to.equal("5");
      expect(String(q.totalYesStake)).to.equal("5");
  
      const a = await fetchAnswer(hash, proposer.publicKey);
      expect(a.answerer.toBase58()).to.equal(proposer.publicKey.toBase58());
      expect(a.side).to.equal(true);
      expect(String(a.stake)).to.equal("5");
      expect(a.claimed).to.equal(false);
    });
  
    it("closes proposers after the answer window (optimistic resolve)", async () => {
      await timeTravel(3605);
  
      const ix = buildCloseProposers({
        cranker: cranker.publicKey,
        hash,
      });
      await sendIx(ix, cranker);
  
      const q = await fetchQuestion(hash);
      expect(q.state).to.deep.equal({ resolved: {} });
  
      const o = await fetchOutcome(hash);
      expect(o.result).to.equal(true);
      expect(o.resolver).to.deep.equal({ optimistic: {} });
      expect(Number(o.answerCount)).to.equal(1);
    });
  
    it("settles the question after the dispute window", async () => {
      await timeTravel(10202);
  
      const ix = buildSettleQuestion({
        cranker: cranker.publicKey,
        hash,
        treasury: TREASURY_PK,
        hasDispute: false,
        hasCouncil: false,
      });
      await sendIx(ix, cranker);
  
      const q = await fetchQuestion(hash);
      expect(q.state).to.deep.equal({ settled: {} });
      expect(Number(q.perAnswerReward)).to.equal(65);
    });
  
    it("winning proposer claims the reward", async () => {
      const pre = await tokenBalance(ataFor(proposer.publicKey));

      const ix = buildClaimReward({
        claimer: proposer.publicKey,
        hash,
        isDisputer: false,
        isCouncilVoter: false,
      });
      await sendIx(ix, proposer);
  
      const post = await tokenBalance(ataFor(proposer.publicKey));
      const expectedPayout = 5n + 65n;
      expect(post - pre).to.equal(expectedPayout);
  
      // answer account closed by claim reward instruction
      expect(await accountExists(answerPdaFor(hash, proposer.publicKey))).to.equal(false);
  
      const rep = await fetchReputation(proposer.publicKey);
      expect(Number(rep.score)).to.equal(10);
      expect(Number(rep.answered)).to.equal(1);
      expect(Number(rep.correct)).to.equal(1);
      expect(Number(rep.timesSlashed)).to.equal(0);
    });
  
    it("closes the question after the close grace period", async () => {
      await timeTravel(86402);

      const ix = buildCloseQuestion({
        cranker: cranker.publicKey,
        creator: creator.publicKey,
        hash,
        treasury: TREASURY_PK,
      });
      await sendIx(ix, cranker);
  
      expect(await accountExists(questionPdaFor(hash))).to.equal(false);
      expect(await accountExists(poolAta)).to.equal(false);
    });
  });
  
  describe("callback (callback_example program receives cassie settle result)", () => {
    it("fires the callback to the callback_example program on settle", async () => {
      const hash = new Uint8Array(crypto.randomBytes(32));
      const creator = newKeypair();
      const proposer = newKeypair();
      const cranker = newKeypair();
      await airdrop(creator.publicKey, 1n);
      await airdrop(proposer.publicKey, 1n);
      await airdrop(cranker.publicKey, 1n);
      // deterministicKeypair(1/2) are reused from the happy path — refund USDC.
      await surfnetFundUsdc(creator.publicKey, 100_000n);
      await surfnetFundUsdc(proposer.publicKey, 100_000n);

      const callbackDisc = HANDLE_CASSIE_RESULT_DISCRIMINATOR_BUF;

      const askIx = buildAsk({
        questioner: creator.publicKey,
        hash,
        bounty: 70n,
        category: 0x62,
        callbackProgram: CALLBACK_EXAMPLE_PROGRAM_ID,
        callbackDiscriminator: callbackDisc,
      });
      await sendIx(askIx, creator);

      const proposeIx = buildPropose({
        proposer: proposer.publicKey,
        hash,
        side: true,
        stake: MIN_STAKE,
      });
      await sendIx(proposeIx, proposer);

      await timeTravel(3605);

      const closeIx = buildCloseProposers({ cranker: cranker.publicKey, hash });
      await sendIx(closeIx, cranker);

      await timeTravel(7501);

      const questionPda = questionPdaFor(hash);
      const settleIx = buildSettleQuestion({
        cranker: cranker.publicKey,
        hash,
        treasury: TREASURY_PK,
        hasDispute: false,
        hasCouncil: false,
        callbackProgram: CALLBACK_EXAMPLE_PROGRAM_ID,
        callbackRemainingAccounts: [
          { pubkey: questionPda, isSigner: false, isWritable: true },
        ],
      });
      await sendIx(settleIx, cranker);
  
      const q = await fetchQuestion(hash);
      expect(q.callbackProgram.toBase58()).to.equal(CALLBACK_EXAMPLE_PROGRAM_ID.toBase58());
      expect(q.state).to.deep.equal({ settled: {} });
    });
  });
});