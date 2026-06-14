import { PublicKey } from "@solana/web3.js";
import { Program, BN, IdlAccounts, IdlTypes } from "@anchor-lang/core";
import { getConnection, getProgram } from "./setup";
import {
  PROGRAM_ID,
  USDC_MINT,
  TOKEN_PROGRAM_ID,
  HANDLE_CASSIE_RESULT_DISCRIMINATOR,
} from "../config";
import {
  questionPdaFor,
  outcomePdaFor,
  disputePdaFor,
  councilTotalPdaFor,
  councilVotePdaFor,
  answerPdaFor,
  reputationPdaFor,
  adminConfigPda,
} from "./instructions";

export type CassieQuestion = any;
export type CassieOracleConfig = any;
export type CassieOutcome = any;
export type CassieAnswer = any;
export type CassieDispute = any;
export type CassieCouncilTotal = any;
export type CassieCouncilVote = any;
export type CassieReputation = any;

async function fetchAccount<T>(name: string, pda: PublicKey): Promise<T> {
  const program = getProgram();
  const account = await getConnection().getAccountInfo(pda);
  if (!account) {
    throw new Error(`Account ${name} (${pda.toBase58()}) not found`);
  }
  const coder = program.coder.accounts;
  const decoded = coder.decode(name, account.data);
  if (!decoded) {
    throw new Error(`Failed to decode account ${name} (${pda.toBase58()})`);
  }
  return decoded as T;
}

export function fetchQuestion(hash: Uint8Array): Promise<CassieQuestion> {
  return fetchAccount<CassieQuestion>("question", questionPdaFor(hash));
}

export async function tryFetchQuestion(hash: Uint8Array): Promise<CassieQuestion | null> {
  try {
    return await fetchQuestion(hash);
  } catch {
    return null;
  }
}

export function fetchOutcome(hash: Uint8Array): Promise<CassieOutcome> {
  return fetchAccount<CassieOutcome>("outcome", outcomePdaFor(hash));
}

export function fetchOracleConfig(): Promise<CassieOracleConfig> {
  return fetchAccount<CassieOracleConfig>("oracleConfig", adminConfigPda());
}

export function fetchAnswer(hash: Uint8Array, proposer: PublicKey): Promise<CassieAnswer> {
  return fetchAccount<CassieAnswer>("answer", answerPdaFor(hash, proposer));
}

export async function tryFetchAnswer(
  hash: Uint8Array,
  proposer: PublicKey
): Promise<CassieAnswer | null> {
  try {
    return await fetchAnswer(hash, proposer);
  } catch {
    return null;
  }
}

export function fetchDispute(hash: Uint8Array): Promise<CassieDispute> {
  return fetchAccount<CassieDispute>("disputeConfig", disputePdaFor(hash));
}

export async function tryFetchDispute(hash: Uint8Array): Promise<CassieDispute | null> {
  try {
    return await fetchDispute(hash);
  } catch {
    return null;
  }
}

export function fetchCouncilTotal(hash: Uint8Array): Promise<CassieCouncilTotal> {
  return fetchAccount<CassieCouncilTotal>("councilTotal", councilTotalPdaFor(hash));
}

export async function tryFetchCouncilTotal(
  hash: Uint8Array
): Promise<CassieCouncilTotal | null> {
  try {
    return await fetchCouncilTotal(hash);
  } catch {
    return null;
  }
}

export function fetchCouncilVote(
  hash: Uint8Array,
  voter: PublicKey
): Promise<CassieCouncilVote> {
  return fetchAccount<CassieCouncilVote>("councilVote", councilVotePdaFor(hash, voter));
}

export async function tryFetchCouncilVote(
  hash: Uint8Array,
  voter: PublicKey
): Promise<CassieCouncilVote | null> {
  try {
    return await fetchCouncilVote(hash, voter);
  } catch {
    return null;
  }
}

export function fetchReputation(owner: PublicKey): Promise<CassieReputation> {
  return fetchAccount<CassieReputation>("reputation", reputationPdaFor(owner));
}

export async function tryFetchReputation(owner: PublicKey): Promise<CassieReputation | null> {
  try {
    return await fetchReputation(owner);
  } catch {
    return null;
  }
}

export async function accountExists(pda: PublicKey): Promise<boolean> {
  const info = await getConnection().getAccountInfo(pda);
  return !!info && info.lamports > 0;
}
