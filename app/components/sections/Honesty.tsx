"use client";

import { Section, SectionTitle } from "@/components/ui/section";
import { Reveal, RevealGroup, RevealItem } from "@/components/ui/reveal";

const STEPS = [
  {
    k: "01",
    head: "Lying posts a bond.",
    body: "To assert a falsehood, you stake capital up front. Your claim is only as cheap as it is true.",
  },
  {
    k: "02",
    head: "Anyone can take it.",
    body: "A correct dispute pays the challenger your bond. Every false answer is a standing bounty for the honest.",
  },
  {
    k: "03",
    head: "So no one tries.",
    body: "When the expected cost of lying exceeds the gain, rational actors stay honest. Disputes stay rare by design.",
  },
];

export function Honesty() {
  return (
    <Section id="honesty">
      <Reveal>
        <SectionTitle className="max-w-3xl">
          Cassie never asks you to trust a person.
          <br />
          It makes lying a <span className="emph">losing trade.</span>
        </SectionTitle>
        <p className="mt-5 max-w-2xl font-mono text-[14px] leading-[1.7] text-parchment-70">
          The protocol carries no opinion of its own. It only arranges the incentives so
          that the truth is always the cheapest answer to give.
        </p>
      </Reveal>

      <RevealGroup className="mt-14 grid gap-px overflow-hidden rounded-md border border-parchment-08 bg-parchment-08 md:grid-cols-3">
        {STEPS.map((s) => (
          <RevealItem key={s.k}>
            <div className="h-full bg-void px-7 py-9">
              <span className="font-ui text-[13px] font-bold tracking-[0.2em] text-amber">
                {s.k}
              </span>
              <h3 className="mt-5 font-display text-[26px] leading-tight text-parchment">
                {s.head}
              </h3>
              <p className="mt-3 font-mono text-[13px] leading-[1.7] text-parchment-55">
                {s.body}
              </p>
            </div>
          </RevealItem>
        ))}
      </RevealGroup>

      <Reveal delay={0.05}>
        <p className="mt-12 max-w-3xl font-display text-[26px] leading-[1.3] text-parchment-70 sm:text-[30px]">
          The oracle is quiet not because no one is watching —{" "}
          <span className="text-parchment">because everyone is.</span>
        </p>
      </Reveal>
    </Section>
  );
}
