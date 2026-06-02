"use client";

import { Section, SectionTitle } from "@/components/ui/section";
import { Reveal } from "@/components/ui/reveal";

const FLOW = [
  { label: "User / Protocol", sub: "posts a question via CPI" },
  { label: "Cassie Program", sub: "Solana · Anchor / Rust" },
  { label: "Resolution Engine", sub: "routes by dispute state", branch: true },
  { label: "On-chain Answer", sub: "verified · final" },
];

const BRANCHES = [
  "Optimistic Settle — no dispute",
  "Weighted Aggregation — disputed",
  "Council Vote — threshold breach",
];

const SPECS = [
  "Written in Anchor (Rust) — fully open source",
  "No trusted admin keys — permissionless deployment",
  "Integrates with any Solana program via CPI",
  "Bonding in SOL or configurable SPL tokens",
  "SDK in TypeScript (npm) and Rust (crate)",
];

export function Architecture() {
  return (
    <Section id="architecture">
      <Reveal>
        <SectionTitle className="max-w-2xl">
          Architecture, <span className="emph">briefly.</span>
        </SectionTitle>
      </Reveal>

      <div className="mt-14 grid gap-10 lg:grid-cols-2">
        {/* flow diagram */}
        <Reveal delay={0.05}>
          <div className="card-verdict p-6 font-mono text-[12.5px]">
            {FLOW.map((node, i) => (
              <div key={node.label}>
                <div className="flex items-center gap-3 py-3">
                  <span className="grid h-7 w-7 shrink-0 place-items-center rounded border border-parchment-15 text-amber">
                    {i + 1}
                  </span>
                  <div>
                    <p className="text-parchment">{node.label}</p>
                    <p className="text-parchment-40">{node.sub}</p>
                  </div>
                </div>

                {node.branch && (
                  <div className="ml-3 border-l border-parchment-15 pl-5">
                    {BRANCHES.map((b) => (
                      <p key={b} className="py-1.5 text-parchment-55">
                        <span className="text-amber">├─</span> {b}
                      </p>
                    ))}
                  </div>
                )}

                {i < FLOW.length - 1 && (
                  <div className="ml-[13px] h-4 border-l border-parchment-15" />
                )}
              </div>
            ))}
          </div>
        </Reveal>

        {/* spec list */}
        <Reveal delay={0.12}>
          <ul className="space-y-px overflow-hidden rounded-md border border-parchment-08">
            {SPECS.map((s) => (
              <li
                key={s}
                className="flex items-start gap-3 bg-void px-5 py-4 font-mono text-[13.5px] text-parchment-70"
              >
                <span className="mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-amber" />
                {s}
              </li>
            ))}
          </ul>
        </Reveal>
      </div>
    </Section>
  );
}
