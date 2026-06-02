"use client";

import { Section, SectionTitle } from "@/components/ui/section";
import { Reveal, RevealGroup, RevealItem } from "@/components/ui/reveal";

const STEPS = [
  {
    n: "①",
    title: "Post",
    desc: "A natural-language claim enters the ledger with a bounty — bait for anyone who can prove otherwise.",
  },
  {
    n: "②",
    title: "Answer",
    desc: "Anyone stakes a bond to assert the answer. No whitelist. No permission.",
  },
  {
    n: "③",
    title: "Window",
    desc: "A dispute window opens. Most claims pass untouched — honesty is the cheap path.",
  },
  {
    n: "④",
    title: "Settle",
    desc: "No valid challenge filed? The claim settles as truth, automatically.",
  },
];

export function HowItWorks() {
  return (
    <Section id="how">
      <Reveal>
        <SectionTitle className="max-w-3xl">
          Optimistic by default. <span className="emph">Honest by design.</span>
        </SectionTitle>
        <p className="mt-5 max-w-2xl font-mono text-[14px] leading-[1.7] text-parchment-70">
          Happy path costs almost nothing. Gaming it costs everything.
        </p>
      </Reveal>

      {/* timeline */}
      <RevealGroup className="mt-16 grid gap-px overflow-hidden rounded-md border border-parchment-08 bg-parchment-08 md:grid-cols-4">
        {STEPS.map((s, i) => (
          <RevealItem key={s.title}>
            <div className="relative h-full bg-void px-6 py-8">
              <div className="flex items-baseline justify-between">
                <span className="font-display text-3xl text-amber">{s.n}</span>
                <span className="font-mono text-[11px] text-parchment-40">
                  0{i + 1}
                </span>
              </div>
              <p className="mt-5 font-ui text-[15px] font-bold uppercase tracking-[0.1em] text-parchment">
                {s.title}
              </p>
              <p className="mt-2 font-mono text-[13px] leading-[1.6] text-parchment-55">
                {s.desc}
              </p>
              {/* connector dot */}
              <span className="absolute right-0 top-1/2 hidden h-2 w-2 -translate-y-1/2 translate-x-1 rounded-full bg-amber/60 md:block last:hidden" />
            </div>
          </RevealItem>
        ))}
      </RevealGroup>

      {/* callouts */}
      <div className="mt-12 grid gap-5 lg:grid-cols-2">
        <Reveal delay={0.05}>
          <div className="card-verdict h-full border-l-2 border-l-amber px-6 py-6">
            <p className="font-ui text-[12px] font-bold uppercase tracking-[0.14em] text-amber">
              The Optimistic Guarantee
            </p>
            <p className="mt-3 font-mono text-[13.5px] leading-[1.7] text-parchment-70">
              In the happy path — an honest answer posted, no one disputes — the question
              settles automatically with near-zero overhead. Cassie only gets expensive
              when someone tries to cheat.
            </p>
          </div>
        </Reveal>

        <Reveal delay={0.12}>
          <div className="card-verdict h-full border-l-2 border-l-crimson px-6 py-6">
            <p className="font-ui text-[12px] font-bold uppercase tracking-[0.14em] text-crimson">
              Dispute Mechanics
            </p>
            <p className="mt-3 font-mono text-[13.5px] leading-[1.7] text-parchment-70">
              Any party can challenge an answer by staking a counter-bond. A dispute
              escalates the question to weighted aggregation. Losers of the dispute
              forfeit their bond to the winning side.
            </p>
          </div>
        </Reveal>
      </div>
    </Section>
  );
}
