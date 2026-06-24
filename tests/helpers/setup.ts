import {
  AnchorProvider,
  Program,
  Idl,
  Wallet,
  BN,
  web3,
} from "@anchor-lang/core";
import * as fs from "fs";
import {
  Connection,
  Keypair,
  PublicKey,
  Commitment,
  Transaction,
  TransactionInstruction,
  TransactionSignature,
  sendAndConfirmTransaction,
  ConfirmOptions,
} from "@solana/web3.js";
import { IDL_PATH, PROGRAM_ID, DEFAULT_CLUSTER_URL, DEFAULT_WALLET_PATH, expandHome } from "../config";

export interface CassieIdl extends Idl {
  address: string;
  instructions: any[];
  accounts?: any[];
  types?: any[];
  events?: any[];
  errors?: any[];
  constants?: any[];
}

let _provider: AnchorProvider | null = null;
let _program: Program | null = null;
let _idl: CassieIdl | null = null;

export function loadIdl(path: string = IDL_PATH): CassieIdl {
  if (_idl && _idl.metadata?.name === "cassie") return _idl;
  const raw = fs.readFileSync(path, "utf-8");
  _idl = JSON.parse(raw) as CassieIdl;
  return _idl;
}

export function loadCallbackIdl(path: string = IDL_PATH): CassieIdl {
  const callbackIdlPath = path.replace("cassie.json", "callback_example.json");
  const raw = fs.readFileSync(callbackIdlPath, "utf-8");
  return JSON.parse(raw) as CassieIdl;
}

export function getClusterUrl(): string {
  return process.env.CASSIE_CLUSTER_URL || DEFAULT_CLUSTER_URL;
}

export function getWalletPath(): string {
  return expandHome(process.env.CASSIE_WALLET_PATH || DEFAULT_WALLET_PATH);
}

export function getProvider(commitment: Commitment = "confirmed"): AnchorProvider {
  if (_provider) return _provider;
  const connection = new Connection(getClusterUrl(), commitment);
  const walletKeypair = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(getWalletPath(), "utf-8")))
  );
  const wallet = new Wallet(walletKeypair);
  _provider = new AnchorProvider(connection, wallet, { commitment, preflightCommitment: commitment });
  return _provider;
}

export function getProgram(): Program {
  if (_program) return _program;
  const idl = loadIdl();
  _program = new Program(idl as Idl, getProvider());
  return _program;
}

export function getConnection(): Connection {
  return getProvider().connection;
}

export function getWallet(): Wallet {
  return getProvider().wallet as Wallet;
}

export function newKeypair(): Keypair {
  return Keypair.generate();
}

export async function airdrop(
  to: PublicKey,
  sol: number | bigint = 10
): Promise<void> {
  const connection = getConnection();
  const lamports = BigInt(sol) * 1_000_000_000n;
  const sig = await connection.requestAirdrop(to, Number(lamports));
  const latestBlockhash = await connection.getLatestBlockhash();
  await connection.confirmTransaction(
    { signature: sig, blockhash: latestBlockhash.blockhash, lastValidBlockHeight: latestBlockhash.lastValidBlockHeight },
    "confirmed"
  );
}

export async function airdropKeypair(
  kp: Keypair,
  sol: number | bigint = 10
): Promise<Keypair> {
  await airdrop(kp.publicKey, sol);
  return kp;
}

export function findPda(seeds: (Buffer | Uint8Array)[]): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(seeds, PROGRAM_ID);
}

export function pda(seeds: (Buffer | Uint8Array)[]): PublicKey {
  return findPda(seeds)[0];
}

export function seedBuf(s: string): Buffer {
  return Buffer.from(s, "utf-8");
}

export async function getBlockTime(): Promise<number> {
  const slot = await getConnection().getSlot();
  return (await getConnection().getBlockTime(slot)) || 0;
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export interface SendIxOptions {
  signers?: Keypair[];
  computeUnits?: number;
  skipPreflight?: boolean;
}

export async function sendIx(
  ix: TransactionInstruction,
  feePayer?: Keypair,
  options: SendIxOptions = {}
): Promise<TransactionSignature> {
  const provider = getProvider();
  const connection = provider.connection;
  const payer = feePayer || (provider.wallet as Wallet).payer;
  const signers = options.signers || [];
  const tx = new Transaction();
  tx.add(ix);
  const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();
  tx.recentBlockhash = blockhash;
  tx.feePayer = payer.publicKey;
  const confirmOptions: ConfirmOptions = {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
    skipPreflight: options.skipPreflight,
  };
  if (signers.length > 0) {
    tx.sign(...signers, payer);
  } else {
    tx.sign(payer);
  }
  try {
    return await sendAndConfirmTransaction(connection, tx, [payer, ...signers], confirmOptions);
  } catch (e: any) {
    if (e?.logs) {
      for (const l of e.logs) console.log("LOG:", l);
    }
    throw e;
  }
}

export async function sendIxs(
  ixs: TransactionInstruction[],
  feePayer?: Keypair,
  options: SendIxOptions = {}
): Promise<TransactionSignature> {
  const provider = getProvider();
  const connection = provider.connection;
  const payer = feePayer || (provider.wallet as Wallet).payer;
  const signers = options.signers || [];
  const tx = new Transaction();
  for (const ix of ixs) tx.add(ix);
  const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();
  tx.recentBlockhash = blockhash;
  tx.feePayer = payer.publicKey;
  if (signers.length > 0) {
    tx.sign(...signers, payer);
  } else {
    tx.sign(payer);
  }
  const confirmOptions: ConfirmOptions = {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
    skipPreflight: options.skipPreflight,
  };
  try {
    return await sendAndConfirmTransaction(connection, tx, [payer, ...signers], confirmOptions);
  } catch (e: any) {
    if (e?.logs) {
      for (const l of e.logs) console.log("LOG:", l);
    }
    throw e;
  }
}

export async function expectIxSuccess(
  promise: Promise<TransactionSignature>
): Promise<TransactionSignature> {
  try {
    return await promise;
  } catch (e: any) {
    if (e?.message) console.error("TX FAILED:", e.message);
    throw e;
  }
}

export async function expectIxFail(
  promise: Promise<TransactionSignature>
): Promise<any> {
  try {
    await promise;
    throw new Error("Expected instruction to fail but it succeeded");
  } catch (e: any) {
    if (e?.message?.includes("Expected instruction to fail")) throw e;
    return e;
  }
}

export function bn(n: number | bigint): BN {
  if (typeof n === "bigint") return new BN(n.toString());
  return new BN(n);
}

export async function timeTravel(seconds: number): Promise<void> {
  const url = getClusterUrl();
  const currentClock = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "getAccountInfo",
      params: ["SysvarC1ock11111111111111111111111111111111", { encoding: "jsonParsed" }],
    }),
  });
  const clockData = await currentClock.json();
  const currentSlot = clockData.result?.context?.slot || 0;
  const target = currentSlot + Math.ceil(seconds / 0.4);
  const body = JSON.stringify({
    jsonrpc: "2.0",
    id: 1,
    method: "surfnet_timeTravel",
    params: [{ absoluteSlot: target }],
  });
  await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body,
  });
}

export function deterministicKeypair(index: number): Keypair {
  const seed = new Uint8Array(32);
  for (let i = 0; i < 32; i++) seed[i] = (index * 17 + i * 31) & 0xff;
  return Keypair.fromSeed(seed);
}

export { BN, web3 };
