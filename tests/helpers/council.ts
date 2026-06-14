import { Keypair, PublicKey } from "@solana/web3.js";
import { airdropKeypair, newKeypair } from "./setup";
import { DEFAULT_AIRDROP_SOL } from "../config";

export async function createCouncil(n: number = 9): Promise<Keypair[]> {
  const members: Keypair[] = [];
  for (let i = 0; i < n; i++) {
    const kp = newKeypair();
    await airdropKeypair(kp, DEFAULT_AIRDROP_SOL);
    members.push(kp);
  }
  return members;
}

export function councilPubkeys(members: Keypair[]): PublicKey[] {
  return members.map((m) => m.publicKey);
}

export function toCouncilArray(members: Keypair[]): PublicKey[] {
  const arr: PublicKey[] = new Array(9).fill(PublicKey.default);
  members.forEach((m, i) => {
    arr[i] = m.publicKey;
  });
  return arr;
}

export async function fund(keypair: Keypair, sol: bigint = DEFAULT_AIRDROP_SOL): Promise<Keypair> {
  return airdropKeypair(keypair, sol);
}
