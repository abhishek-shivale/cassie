import { PublicKey } from "@solana/web3.js";
import * as path from "path";

export const PROGRAM_ID = new PublicKey("8XBYSkbwTEonoFRtqaU8PqbwyXaXvzDT1bApyUdRbrwf");

export const CALLBACK_EXAMPLE_PROGRAM_ID = new PublicKey(
  "DANGHof54KqrvGnipP3Hm8whXXifmaWKQwYYH533jVaq"
);

export const USDC_MINT = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
export const TOKEN_PROGRAM_ID = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
export const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey(
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
);
export const SYSTEM_PROGRAM_ID = new PublicKey("11111111111111111111111111111111");
// export const MEMO_PROGRAM_ID = new PublicKey("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");

export const TOKEN_DECIMALS = 6;

// Fixed test accounts for deterministic testing
export const ASKER = new PublicKey("C4nMqhtwcegrTzRVNra6Us7eYgyTag7QWzij3PH9QsiY");
export const ASKER_ATA = new PublicKey("BUc8ghemRNA5y24ciN1mm9mTrr8ceDYtrxSrizW2XeAm");

export const PROPOSER = new PublicKey("28NjShMmvmzfdFHhJQ9v2MGRcVNgwqFNnE8xjCFP1ZL9");
export const PROPOSER_ATA = new PublicKey("FaSASyhrmw7WiLQRuHkifB9sGmiDgBeU7tGzp7zGuKUm");

export const CRANKER = new PublicKey("Gtc1rXuattKQ8qDB8943sNdWD8yEsi2yF3G2RSD1kjkW");
export const CRANKER_ATA = new PublicKey("Gyrm61f3HhR8PGx7wwksNGQbAvuevqPF8BSi8J4xhgay");

export const TREASURY_PK = new PublicKey("4DYBTqH2CAPQNvHhyvqSHicQp7qKaTpSHvSrzKW4GCgA");
export const TREASURY_ATA = new PublicKey("EXwWh334eU5BTtExJVqr2eXWkByfmcxAE4m59hDoNtZU");

export const COUNCIL_PKS: PublicKey[] = [
  new PublicKey("BwSjPDk4eAijSKeVPMPrN2paxvAC5tD67BkH4RyqydRD"),
  new PublicKey("Bjqcds16kesBp8DzhAaf1z6FVR86Xr3cb29o6FNTy87w"),
  new PublicKey("67xR64KfrN8Mo3RBn7FKedZUxaqmFPtCctZAUT3b3fmL"),
  new PublicKey("8TmRUKKuVQDtwFKP2z7LXBEbGmz5jxVTEbRvtGVv41Ah"),
  new PublicKey("A7WmaYhwrvHUNBXA5UHdi6LMRwhqnwCJGH4SFpV4uQPF"),
  new PublicKey("543oEg8CDHZh7UWhwv1zsZ5qx4vRYLd8FQEpx6qeyjRz"),
  new PublicKey("ESjYsYdpAcDj5Dv1TSfeFSZECqjJfF62ZN1EBrWTPQpC"),
  new PublicKey("6ZZ8X5ZwQ6kS9WywdrFChi2WabbbRjdfCBVnyc1tES1F"),
  new PublicKey("6AMRXBLoAKv8MKZTqNiKwnzc9ztXpH8qUETfZw2W4pTs"),
];

export const SLASH_BPS = 5000;
export const TREASURY_BPS = 750;
export const COUNCIL_BPS = 1500;
export const DIVERGENCE_BPS = 3500;

export const MIN_BOUNTY = 10n;
export const MIN_STAKE = 750n;
export const MIN_DISPUTE_BOND = 750n;
export const BPS_DENOMINATOR = 10_000n;

export const SECONDS_PER_DAY = 86_400;
export const DEFAULT_ANSWER_WINDOW = 3600;
export const DEFAULT_DISPUTE_WINDOW = 7200;
export const DEFAULT_COUNCIL_WINDOW = SECONDS_PER_DAY;
export const CLOSE_GRACE = SECONDS_PER_DAY;

export const ONE_SOL = 1_000_000_000n;
export const DEFAULT_AIRDROP_SOL = 10n;

export const IDL_PATH = path.join(__dirname, "..", "target", "idl", "cassie.json");
export const CALLBACK_IDL_PATH = path.join(
  __dirname,
  "..",
  "target",
  "idl",
  "callback_example.json"
);

export const SEED_ADMIN_CONFIG = "admin_config";
export const SEED_QUESTION_CONFIG = "question_config";
export const SEED_ANSWER = "answer";
export const SEED_REPUTATION = "reputation";
export const SEED_OUTCOME = "outcome";
export const SEED_COUNCIL_TOTAL = "council_total";
export const SEED_COUNCIL_VOTE = "council_vote";
export const SEED_DISPUTE = "dispute";

export const HANDLE_CASSIE_RESULT_DISCRIMINATOR = Buffer.from([
  53, 164, 71, 40, 180, 130, 198, 236,
]);

export const DEFAULT_CLUSTER_URL = "http://127.0.0.1:8899";
export const DEFAULT_WALLET_PATH = "~/.config/solana/id.json";

export function expandHome(p: string): string {
  return p.startsWith("~") ? path.join(process.env.HOME || "", p.slice(1)) : p;
}
