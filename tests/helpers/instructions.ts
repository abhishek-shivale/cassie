import {
  AccountMeta,
  PublicKey,
  TransactionInstruction,
  Keypair,
} from "@solana/web3.js";
import { BN, Program, Idl } from "@anchor-lang/core";
import {
  PROGRAM_ID,
  SEED_ADMIN_CONFIG,
  SEED_QUESTION_CONFIG,
  SEED_ANSWER,
  SEED_REPUTATION,
  SEED_OUTCOME,
  SEED_COUNCIL_TOTAL,
  SEED_COUNCIL_VOTE,
  SEED_DISPUTE,
  SLASH_BPS,
  TREASURY_BPS,
  COUNCIL_BPS,
  DIVERGENCE_BPS,
  MIN_BOUNTY,
  DEFAULT_ANSWER_WINDOW,
  DEFAULT_DISPUTE_WINDOW,
  DEFAULT_COUNCIL_WINDOW,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  SYSTEM_PROGRAM_ID,
  USDC_MINT,
  HANDLE_CASSIE_RESULT_DISCRIMINATOR,
} from "../config";
import { findPda, pda, getProgram } from "./setup";
import { ataFor } from "./tokens";

export interface InitializeConfigArgs {
  admin: PublicKey;
  treasury: PublicKey;
  council: PublicKey[];
  councilSize?: number;
  defaultAnswerWindow?: BN | number;
  defaultDisputeWindow?: BN | number;
  defaultCouncilWindow?: BN | number;
  divergenceBps?: number;
  minBounty?: BN | number;
  slashBps?: number;
  treasuryBps?: number;
  councilBps?: number;
}

export function buildInitializeConfig(args: InitializeConfigArgs): TransactionInstruction {
  const councilArr: PublicKey[] = new Array(9).fill(PublicKey.default);
  for (let i = 0; i < args.council.length; i++) councilArr[i] = args.council[i];
  const program = getProgram();
  return program.instruction.initializeConfig(
    bnTo(args.defaultAnswerWindow ?? DEFAULT_ANSWER_WINDOW),
    bnTo(args.defaultCouncilWindow ?? DEFAULT_COUNCIL_WINDOW),
    bnTo(args.defaultDisputeWindow ?? DEFAULT_DISPUTE_WINDOW),
    bnTo(args.divergenceBps ?? DIVERGENCE_BPS),
    bnTo(args.minBounty ?? MIN_BOUNTY),
    bnTo(args.slashBps ?? SLASH_BPS),
    args.treasury,
    bnTo(args.treasuryBps ?? TREASURY_BPS),
    bnTo(args.councilBps ?? COUNCIL_BPS),
    councilArr,
    args.councilSize ?? 9,
    {
      accounts: {
        admin: args.admin,
        config: pda([Buffer.from(SEED_ADMIN_CONFIG)]),
        usdcMint: USDC_MINT,
        systemProgram: SYSTEM_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    }
  ) as TransactionInstruction;
}

export interface UpdateConfigArgs {
  admin: PublicKey;
  defaultDisputeWindow?: BN | number | null;
  defaultCouncilWindow?: BN | number | null;
  defaultAnswerWindow?: BN | number | null;
  freeze?: boolean | null;
}

export function buildUpdateConfig(args: UpdateConfigArgs): TransactionInstruction {
  const program = getProgram();
  return program.instruction.updateConfig(
    args.defaultDisputeWindow == null ? null : bnTo(args.defaultDisputeWindow),
    args.defaultCouncilWindow == null ? null : bnTo(args.defaultCouncilWindow),
    args.defaultAnswerWindow == null ? null : bnTo(args.defaultAnswerWindow),
    args.freeze == null ? null : args.freeze,
    {
      accounts: {
        admin: args.admin,
        config: pda([Buffer.from(SEED_ADMIN_CONFIG)]),
      },
    }
  ) as TransactionInstruction;
}

export interface UpdateCouncilArgs {
  admin: PublicKey;
  old: PublicKey;
  new: PublicKey;
}

export function buildUpdateCouncil(args: UpdateCouncilArgs): TransactionInstruction {
  const program = getProgram();
  return program.instruction.updateCouncil(args.old, args.new, {
    accounts: {
      admin: args.admin,
      config: pda([Buffer.from(SEED_ADMIN_CONFIG)]),
    },
  }) as TransactionInstruction;
}

export interface AskArgs {
  questioner: PublicKey;
  hash: Uint8Array;
  bounty?: bigint;
  category?: number;
  metadataUri?: Uint8Array;
  callbackProgram?: PublicKey;
  callbackDiscriminator?: Uint8Array;
}

export function buildAsk(args: AskArgs): TransactionInstruction {
  const program = getProgram();
  const hash = Buffer.from(args.hash);
  const questionPda = pda([Buffer.from(SEED_QUESTION_CONFIG), hash]);
  return program.instruction.ask(
    [...hash],
    bnTo(args.bounty ?? 70n),
    args.category ?? 0x62,
    [...(args.metadataUri ?? new Uint8Array(128))],
    args.callbackProgram ?? PublicKey.default,
    [...(args.callbackDiscriminator ?? new Uint8Array(8))],
    {
      accounts: {
        questioner: args.questioner,
        config: pda([Buffer.from(SEED_ADMIN_CONFIG)]),
        question: questionPda,
        usdcMint: USDC_MINT,
        questionerAta: ataFor(args.questioner),
        bountyAta: ataFor(questionPda),
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SYSTEM_PROGRAM_ID,
      },
    }
  ) as TransactionInstruction;
}

export interface ProposeArgs {
  proposer: PublicKey;
  hash: Uint8Array;
  side: boolean;
  stake?: bigint;
}

export function buildPropose(args: ProposeArgs): TransactionInstruction {
  const program = getProgram();
  const hash = Buffer.from(args.hash);
  const questionPda = pda([Buffer.from(SEED_QUESTION_CONFIG), hash]);
  return program.instruction.propose(
    [...hash],
    bnTo(args.stake ?? 750n),
    args.side,
    {
      accounts: {
        proposer: args.proposer,
        question: questionPda,
        config: pda([Buffer.from(SEED_ADMIN_CONFIG)]),
        usdcMint: USDC_MINT,
        proposerAta: ataFor(args.proposer),
        bondAta: ataFor(questionPda),
        reputation: pda([Buffer.from(SEED_REPUTATION), args.proposer.toBuffer()]),
        answer: pda([Buffer.from(SEED_ANSWER), hash, args.proposer.toBuffer()]),
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SYSTEM_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      },
    }
  ) as TransactionInstruction;
}

export interface CloseProposersArgs {
  cranker: PublicKey;
  hash: Uint8Array;
}

export function buildCloseProposers(args: CloseProposersArgs): TransactionInstruction {
  const program = getProgram();
  const hash = Buffer.from(args.hash);
  const questionPda = pda([Buffer.from(SEED_QUESTION_CONFIG), hash]);
  return program.instruction.closeProposers([...hash], {
    accounts: {
      cranker: args.cranker,
      question: questionPda,
      config: pda([Buffer.from(SEED_ADMIN_CONFIG)]),
      outcome: pda([Buffer.from(SEED_OUTCOME), hash]),
      systemProgram: SYSTEM_PROGRAM_ID,
    },
  }) as TransactionInstruction;
}

export interface DisputeArgs {
  disputer: PublicKey;
  hash: Uint8Array;
  bond?: bigint;
  claimedOutcome: boolean;
  reasonHash?: Uint8Array;
}

export function buildDispute(args: DisputeArgs): TransactionInstruction {
  const program = getProgram();
  const hash = Buffer.from(args.hash);
  const questionPda = pda([Buffer.from(SEED_QUESTION_CONFIG), hash]);
  return program.instruction.dispute(
    [...hash],
    bnTo(args.bond ?? 750n),
    args.claimedOutcome,
    [...(args.reasonHash ?? new Uint8Array(128))],
    {
      accounts: {
        disputer: args.disputer,
        question: questionPda,
        usdcMint: USDC_MINT,
        disputerAta: ataFor(args.disputer),
        bondAta: ataFor(questionPda),
        config: pda([Buffer.from(SEED_ADMIN_CONFIG)]),
        outcome: pda([Buffer.from(SEED_OUTCOME), hash]),
        disputerConfig: pda([Buffer.from(SEED_DISPUTE), hash]),
        reputation: pda([Buffer.from(SEED_REPUTATION), args.disputer.toBuffer()]),
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SYSTEM_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      },
    }
  ) as TransactionInstruction;
}

export interface CouncilVoteArgs {
  voter: PublicKey;
  hash: Uint8Array;
  vote: boolean;
  bond?: bigint;
}

export function buildCouncilVote(args: CouncilVoteArgs): TransactionInstruction {
  const program = getProgram();
  const hash = Buffer.from(args.hash);
  const questionPda = pda([Buffer.from(SEED_QUESTION_CONFIG), hash]);
  return program.instruction.councilVote(
    [...hash],
    args.vote,
    bnTo(args.bond ?? 750n),
    {
      accounts: {
        voter: args.voter,
        question: questionPda,
        config: pda([Buffer.from(SEED_ADMIN_CONFIG)]),
        usdcMint: USDC_MINT,
        councilTotal: pda([Buffer.from(SEED_COUNCIL_TOTAL), hash]),
        councilVote: pda([Buffer.from(SEED_COUNCIL_VOTE), hash, args.voter.toBuffer()]),
        reputation: pda([Buffer.from(SEED_REPUTATION), args.voter.toBuffer()]),
        voterAta: ataFor(args.voter),
        rewardPool: ataFor(questionPda),
        systemProgram: SYSTEM_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      },
    }
  ) as TransactionInstruction;
}

export interface FinalizeCouncilArgs {
  cranker: PublicKey;
  hash: Uint8Array;
}

export function buildFinalizeCouncil(args: FinalizeCouncilArgs): TransactionInstruction {
  const program = getProgram();
  const hash = Buffer.from(args.hash);
  return program.instruction.finalizeCouncil([...hash], {
    accounts: {
      cranker: args.cranker,
      question: pda([Buffer.from(SEED_QUESTION_CONFIG), hash]),
      config: pda([Buffer.from(SEED_ADMIN_CONFIG)]),
      councilTotal: pda([Buffer.from(SEED_COUNCIL_TOTAL), hash]),
      outcome: pda([Buffer.from(SEED_OUTCOME), hash]),
    },
  }) as TransactionInstruction;
}

export interface SettleQuestionArgs {
  cranker: PublicKey;
  hash: Uint8Array;
  treasury: PublicKey;
  hasDispute?: boolean;
  hasCouncil?: boolean;
  callbackProgram?: PublicKey | null;
  callbackRemainingAccounts?: AccountMeta[];
}

export function buildSettleQuestion(args: SettleQuestionArgs): TransactionInstruction {
  const program = getProgram();
  const hash = Buffer.from(args.hash);
  const questionPda = pda([Buffer.from(SEED_QUESTION_CONFIG), hash]);
  const ctx: any = {
    accounts: {
      cranker: args.cranker,
      question: questionPda,
      config: pda([Buffer.from(SEED_ADMIN_CONFIG)]),
      outcome: pda([Buffer.from(SEED_OUTCOME), hash]),
      usdcMint: USDC_MINT,
      poolAta: ataFor(questionPda),
      treasuryAta: ataFor(args.treasury),
      tokenProgram: TOKEN_PROGRAM_ID,
      dispute: args.hasDispute ? pda([Buffer.from(SEED_DISPUTE), hash]) : PROGRAM_ID,
      councilTotal: args.hasCouncil ? pda([Buffer.from(SEED_COUNCIL_TOTAL), hash]) : PROGRAM_ID,
      callbackProgram: args.callbackProgram ?? PROGRAM_ID,
    },
    remainingAccounts: args.callbackRemainingAccounts ?? [],
  };
  return program.instruction.settleQuestion([...hash], ctx) as TransactionInstruction;
}

export interface ClaimRewardArgs {
  claimer: PublicKey;
  hash: Uint8Array;
  isDisputer?: boolean;
  isCouncilVoter?: boolean;
}

export function buildClaimReward(args: ClaimRewardArgs): TransactionInstruction {
  const program = getProgram();
  const hash = Buffer.from(args.hash);
  const questionPda = pda([Buffer.from(SEED_QUESTION_CONFIG), hash]);
  const ctx: any = {
    accounts: {
      claimer: args.claimer,
      question: questionPda,
      config: pda([Buffer.from(SEED_ADMIN_CONFIG)]),
      outcome: pda([Buffer.from(SEED_OUTCOME), hash]),
      reputation: pda([Buffer.from(SEED_REPUTATION), args.claimer.toBuffer()]),
      usdcMint: USDC_MINT,
      poolAta: ataFor(questionPda),
      claimerAta: ataFor(args.claimer),
      tokenProgram: TOKEN_PROGRAM_ID,
      dispute: args.isDisputer ? pda([Buffer.from(SEED_DISPUTE), hash]) : PROGRAM_ID,
      councilVote: args.isCouncilVoter ? pda([Buffer.from(SEED_COUNCIL_VOTE), hash, args.claimer.toBuffer()]) : PROGRAM_ID,
      answer: pda([Buffer.from(SEED_ANSWER), hash, args.claimer.toBuffer()]),
    },
  };
  return program.instruction.claimReward([...hash], ctx) as TransactionInstruction;
}

export interface CloseQuestionArgs {
  cranker: PublicKey;
  creator: PublicKey;
  hash: Uint8Array;
  treasury: PublicKey;
  hasCouncil?: boolean;
}

export function buildCloseQuestion(args: CloseQuestionArgs): TransactionInstruction {
  const program = getProgram();
  const hash = Buffer.from(args.hash);
  const questionPda = pda([Buffer.from(SEED_QUESTION_CONFIG), hash]);
  const ctx: any = {
    accounts: {
      cranker: args.cranker,
      creator: args.creator,
      question: questionPda,
      config: pda([Buffer.from(SEED_ADMIN_CONFIG)]),
      outcome: pda([Buffer.from(SEED_OUTCOME), hash]),
      usdcMint: USDC_MINT,
      poolAta: ataFor(questionPda),
      treasuryAta: ataFor(args.treasury),
      tokenProgram: TOKEN_PROGRAM_ID,
      councilTotal: args.hasCouncil ? pda([Buffer.from(SEED_COUNCIL_TOTAL), hash]) : PROGRAM_ID,
    },
  };
  return program.instruction.closeQuestion([...hash], ctx) as TransactionInstruction;
}

export function questionPdaFor(hash: Uint8Array): PublicKey {
  return pda([Buffer.from(SEED_QUESTION_CONFIG), Buffer.from(hash)]);
}

export function outcomePdaFor(hash: Uint8Array): PublicKey {
  return pda([Buffer.from(SEED_OUTCOME), Buffer.from(hash)]);
}

export function disputePdaFor(hash: Uint8Array): PublicKey {
  return pda([Buffer.from(SEED_DISPUTE), Buffer.from(hash)]);
}

export function councilTotalPdaFor(hash: Uint8Array): PublicKey {
  return pda([Buffer.from(SEED_COUNCIL_TOTAL), Buffer.from(hash)]);
}

export function councilVotePdaFor(hash: Uint8Array, voter: PublicKey): PublicKey {
  return pda([Buffer.from(SEED_COUNCIL_VOTE), Buffer.from(hash), voter.toBuffer()]);
}

export function answerPdaFor(hash: Uint8Array, proposer: PublicKey): PublicKey {
  return pda([Buffer.from(SEED_ANSWER), Buffer.from(hash), proposer.toBuffer()]);
}

export function reputationPdaFor(owner: PublicKey): PublicKey {
  return pda([Buffer.from(SEED_REPUTATION), owner.toBuffer()]);
}

export function adminConfigPda(): PublicKey {
  return pda([Buffer.from(SEED_ADMIN_CONFIG)]);
}

export function bnTo(n: number | bigint | BN): BN {
  if (n instanceof BN) return n;
  if (typeof n === "bigint") return new BN(n.toString());
  return new BN(n);
}

export const HANDLE_CASSIE_RESULT_DISCRIMINATOR_BUF = Buffer.from(
  HANDLE_CASSIE_RESULT_DISCRIMINATOR
);
