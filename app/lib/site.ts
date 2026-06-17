
export const site = {
  name: "Cassie",
  tagline: "Permissionless optimistic oracle on Solana",
  links: {
    app: "https://cassie.abhishek.win",
    docs: "https://cassie.abhishek.win",
    github: "https://github.com/abhishek-shivale/cassie",
    x: "https://x.com/abhishekwinn",
    program: "8XBYSkbwTEonoFRtqaU8PqbwyXaXvzDT1bApyUdRbrwf",
  },
} as const;

export const nav: { label: string; href: string }[] = [
];

export const stats = [
  { value: 48213, label: "Questions settled", suffix: "" },
  { value: 12.4, label: "Bonded value", prefix: "$", suffix: "M", decimals: 1 },
  { value: 99.3, label: "Resolved optimistically", suffix: "%", decimals: 1 },
  { value: 1.9, label: "Median settle time", suffix: " min", decimals: 1 },
] as const;

export type FlowStep = {
  id: string;
  index: string;
  title: string;
  role: string;
  blurb: string;
  detail: string;
  accent: "amber" | "teal" | "petrol" | "copper";
};

export const flow: FlowStep[] = [
  {
    id: "ask",
    index: "01",
    title: "Ask",
    role: "Requester",
    blurb: "Post a question with a bounty.",
    detail:
      "Deposit a bounty and post a verifiable YES/NO question on-chain. Optionally register a callback to fire on settlement.",
    accent: "amber",
  },
  {
    id: "propose",
    index: "02",
    title: "Propose",
    role: "Proposer",
    blurb: "Bond behind an answer.",
    detail:
      "Anyone stakes a bond asserting the answer. Unchallenged within the window, it settles and the bond returns with the bounty.",
    accent: "amber",
  },
  {
    id: "dispute",
    index: "03",
    title: "Dispute",
    role: "Challenger",
    blurb: "Stake against a wrong answer.",
    detail:
      "Post a counter-bond claiming the opposite outcome. The optimistic path closes and resolution moves to reputation-weighted voting.",
    accent: "amber",
  },
  {
    id: "council",
    index: "04",
    title: "Council",
    role: "Reputation + Council",
    blurb: "Resolve genuinely contested truth.",
    detail:
      "If no answer holds a clear weighted supermajority, bonded council members vote. Quorum settles the winning side.",
    accent: "amber",
  },
  {
    id: "settle",
    index: "05",
    title: "Settle",
    role: "Protocol",
    blurb: "Pay out, slash, fire the callback.",
    detail:
      "Winning bonds return with reward, wrong bonds are slashed, reputation updates, and a callback CPI fires into any registered program.",
    accent: "amber",
  },
];

export const mechanics = [
  {
    id: "bonding",
    kbd: "BOND",
    title: "Skin in the game",
    body: "Every answer and every dispute is backed by a token bond. Being right returns your stake with reward; being wrong forfeits it to the people who corrected you. Honesty is the profitable move.",
    points: [
      "Symmetric bonds on both sides of a dispute",
      "Slashed stake routed to correct voters",
      "Bounty + bond settle atomically on Solana",
    ],
    accent: "amber" as const,
  },
  {
    id: "reputation",
    kbd: "REP",
    title: "Reputation that compounds",
    body: "Vote with the truth and your reputation grows, weighting your future voice. Vote against settled outcomes and it decays. Influence is earned over time, never bought.",
    points: [
      "Reputation-weighted vote tallies",
      "Earned through accurate resolutions",
      "Sybil-resistant by design, not by gatekeeping",
    ],
    accent: "teal" as const,
  },
];

export const councilPoints = [
  {
    title: "Last resort, not first",
    body: "99% of questions settle optimistically with no vote at all. The council only convenes when reputation-weighted voting cannot cleanly resolve a contested outcome.",
  },
  {
    title: "Transparent and bounded",
    body: "Council membership, mandate, and every ruling are on-chain and auditable. Authority is scoped to disputed edge cases — never to routine answers.",
  },
  {
    title: "Aligned by stake",
    body: "Council members are bonded and accountable. A bad-faith ruling is economically punished, keeping the backstop honest.",
  },
];
