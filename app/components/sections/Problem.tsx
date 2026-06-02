"use client";

import { Section, SectionTitle } from "@/components/ui/section";
import { Reveal } from "@/components/ui/reveal";

const CATEGORIES = [
  { kind: "Price Oracles", names: "Pyth · Chainlink", active: false },
  { kind: "Data Oracles", names: "Switchboard", active: false },
  { kind: "Truth Oracles", names: "Cassie", active: true },
];

export function Problem() {
  return (
    <Section id="problem">
      <Reveal>
        <SectionTitle className="max-w-3xl">
          Why another <span className="emph">oracle?</span>
        </SectionTitle>
      </Reveal>

      <div className="mt-14 grid gap-12 lg:grid-cols-[0.9fr_1.1fr]">
        <Reveal delay={0.05}>
          <div className="space-y-3 font-mono text-[19px] leading-[1.5] text-parchment-70 sm:text-[22px]">
            <p>
              <span className="text-amber">Pyth</span> tells you how much.
            </p>
            <p>
              <span className="text-amber">Switchboard</span> tells you what happened.
            </p>
            <p>
              Neither tells you{" "}
              <span className="text-amber">did it happen?</span>
            </p>
          </div>
        </Reveal>

        <Reveal delay={0.12}>
          <div className="space-y-5 font-mono text-[14px] leading-[1.75] text-parchment-70">
            <p>
              Prediction markets, parametric insurance, and event-driven protocols all
              need answers to arbitrary truth claims. Was the match won? Did the
              earthquake exceed 6.2 magnitude? Was the delivery confirmed?
            </p>
            <p>
              Today, Solana has no native primitive for this. You either trust a
              centralized resolver or bridge a verdict from Ethereum — adding latency,
              trust assumptions, and fees. Cassie was built for exactly this gap.
            </p>
          </div>
        </Reveal>
      </div>

      {/* category ledger */}
      <Reveal delay={0.05}>
        <div className="mt-16 grid gap-px overflow-hidden rounded-md border border-parchment-08 bg-parchment-08 sm:grid-cols-3">
          {CATEGORIES.map((c) => (
            <div
              key={c.kind}
              className={
                "relative bg-void px-6 py-8 " +
                (c.active ? "bg-sage-tint" : "")
              }
            >
              {c.active && (
                <span className="stamp stamp--settled absolute right-4 top-4 rotate-6 text-[10px]">
                  New
                </span>
              )}
              <p
                className={
                  "font-ui text-[15px] font-bold uppercase tracking-[0.1em] " +
                  (c.active ? "text-amber" : "text-parchment-40")
                }
              >
                {c.kind}
              </p>
              <p
                className={
                  "mt-2 font-mono text-[13px] " +
                  (c.active ? "text-parchment" : "text-parchment-40")
                }
              >
                {c.names}
              </p>
            </div>
          ))}
        </div>
      </Reveal>
    </Section>
  );
}
