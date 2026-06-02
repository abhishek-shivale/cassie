const ITEMS: { q: string; status: "settled" | "disputed" | "open" }[] = [
  { q: "Did BTC close above $90,000 on 2026-05-31?", status: "settled" },
  { q: "Earthquake > 6.2 magnitude near Tokyo this week?", status: "disputed" },
  { q: "Was shipment #4471 delivered to the port?", status: "settled" },
  { q: "Did Solana TPS exceed 65,000 on block #312,847,001?", status: "settled" },
  { q: "Flight UA-482 delayed more than 3 hours?", status: "open" },
  { q: "Rainfall in Iowa exceeded 4 inches in May?", status: "settled" },
  { q: "Did the DAO proposal #88 reach quorum?", status: "settled" },
  { q: "Match final: did the home team win?", status: "open" },
];

const LABEL = { settled: "Settled", disputed: "Disputed", open: "Open" } as const;

function Row() {
  return (
    <div className="flex shrink-0">
      {ITEMS.map((it, i) => (
        <div key={i} className="flex items-center gap-4 px-7 py-4">
          <span className={`pill pill--${it.status} shrink-0`}>{LABEL[it.status]}</span>
          <span className="whitespace-nowrap font-mono text-[13px] text-parchment-70">
            {it.q}
          </span>
          <span className="px-2 font-mono text-parchment-15">/</span>
        </div>
      ))}
    </div>
  );
}

export function Ticker() {
  return (
    <div className="marquee hairline-t hairline-b bg-void-raise" aria-hidden>
      <div className="marquee-track">
        <Row />
        <Row />
      </div>
    </div>
  );
}
