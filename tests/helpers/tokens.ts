import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  TransactionInstruction,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import {
  createInitializeMint2Instruction,
  getAssociatedTokenAddressSync,
  MINT_SIZE,
  TOKEN_PROGRAM_ID as SPL_TOKEN_PROGRAM_ID,
  createMintToInstruction,
  createTransferCheckedInstruction,
  getMinimumBalanceForRentExemptMint,
  getAccount,
  getMint,
  ASSOCIATED_TOKEN_PROGRAM_ID as SPL_ATA_PROGRAM_ID,
} from "@solana/spl-token";
import { getConnection, sendIx, sendIxs, getClusterUrl } from "./setup";
import {
  TOKEN_DECIMALS,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  USDC_MINT,
  ONE_SOL,
} from "../config";

export interface MintResult {
  mint: PublicKey;
}

export async function ensureUsdcMint(
  payer: Keypair,
  decimals: number = TOKEN_DECIMALS,
  mint: PublicKey = USDC_MINT
): Promise<PublicKey> {
  const connection = getConnection();
  const existing = await connection.getAccountInfo(mint);
  if (existing) return mint;
  const lamports = await getMinimumBalanceForRentExemptMint(connection);
  const ixs: TransactionInstruction[] = [
    SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: mint,
      space: MINT_SIZE,
      lamports,
      programId: SPL_TOKEN_PROGRAM_ID,
    }),
    createInitializeMint2Instruction(mint, decimals, payer.publicKey, null, SPL_TOKEN_PROGRAM_ID),
  ];
  await sendIxs(ixs, payer);
  return mint;
}

export function ataFor(owner: PublicKey, mint: PublicKey = USDC_MINT): PublicKey {
  return getAssociatedTokenAddressSync(
    mint,
    owner,
    true,
    SPL_TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
}

export async function getOrCreateAta(
  owner: PublicKey,
  payer: Keypair,
  mint: PublicKey = USDC_MINT
): Promise<PublicKey> {
  const ata = ataFor(owner, mint);
  const connection = getConnection();
  const info = await connection.getAccountInfo(ata);
  if (info) return ata;
  const ix = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: ata, isSigner: false, isWritable: true },
      { pubkey: owner, isSigner: false, isWritable: false },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SPL_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SPL_ATA_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    programId: SPL_ATA_PROGRAM_ID,
    data: Buffer.from([0]),
  });
  await sendIx(ix, payer);
  return ata;
}

export async function mintUsdc(
  destination: PublicKey,
  amount: bigint,
  payer: Keypair,
  mint: PublicKey = USDC_MINT
): Promise<string> {
  const ix = createMintToInstruction(
    mint,
    destination,
    payer.publicKey,
    BigInt(amount),
    [],
    SPL_TOKEN_PROGRAM_ID
  );
  return sendIx(ix, payer);
}

export async function tokenBalance(ata: PublicKey): Promise<bigint> {
  const connection = getConnection();
  try {
    const acc = await getAccount(connection, ata, "confirmed", SPL_TOKEN_PROGRAM_ID);
    return acc.amount;
  } catch {
    return 0n;
  }
}

export async function transferUsdc(
  fromAta: PublicKey,
  toAta: PublicKey,
  amount: bigint,
  owner: Keypair,
  mint: PublicKey = USDC_MINT
): Promise<string> {
  const ix = createTransferCheckedInstruction(
    fromAta,
    mint,
    toAta,
    owner.publicKey,
    BigInt(amount),
    TOKEN_DECIMALS,
    [],
    SPL_TOKEN_PROGRAM_ID
  );
  return sendIx(ix, owner);
}

export async function fundOwnerWithUsdc(
  owner: PublicKey,
  amount: bigint,
  payer: Keypair,
  mint: PublicKey = USDC_MINT
): Promise<{ ata: PublicKey }> {
  const ata = await getOrCreateAta(owner, payer, mint);
  const payerAta = ataFor(payer.publicKey, mint);
  await transferUsdc(payerAta, ata, amount, payer, mint);
  return { ata };
}

export function bnReplacer(_k: string, v: any): any {
  if (typeof v === "bigint") return `0x${v.toString(16)}`;
  return v;
}

/**
 * Surfpool (mainnet fork) helper: funds a token account via surfnet_setTokenAccount.
 * Takes the owner pubkey directly — Surfpool derives and creates the ATA internally.
 * Use this instead of ensureUsdcMint + fundOwnerWithUsdc on Surfpool.
 */
export async function surfnetFundUsdc(
  owner: PublicKey,
  amount: bigint,
  mint: PublicKey = USDC_MINT
): Promise<PublicKey> {
  const res = await fetch(getClusterUrl(), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "surfnet_setTokenAccount",
      params: [
        owner.toBase58(),
        mint.toBase58(),
        { amount: Number(amount) },
        SPL_TOKEN_PROGRAM_ID.toBase58(),
      ],
    }),
  });

  const json = await res.json();
  if (json.error) {
    throw new Error(`surfnet_setTokenAccount failed: ${JSON.stringify(json.error)}`);
  }

  return ataFor(owner, mint);
}