"use client";

import { Section, SectionTitle } from "@/components/ui/section";
import { Reveal, RevealGroup, RevealItem } from "@/components/ui/reveal";
import { MarketsIcon, InsuranceIcon, MilestoneIcon } from "@/components/icons/UseCaseIcons";

const CASES = [
  {
    Icon: MarketsIcon,
    title: "Prediction Markets",
    desc: "Settle markets on any real-world event without a centralized resolver. Cassie's truth layer is the missing piece for fully on-chain prediction protocols.",
  },
  {
    Icon: InsuranceIcon,
    title: "Parametric Insurance",
    desc: "Trigger automatic payouts when a condition is provably met — earthquake magnitude, flight delay, crop failure. No claims adjuster. No approval queue.",
  },
  {
    Icon: MilestoneIcon,
    title: "Event-Driven Protocols",
    desc: "Unlock funds, mint tokens, or trigger governance when an off-chain event occurs. Any protocol that reacts to reality can use Cassie as its condition layer.",
  },
];

export function UseCases() {
  return (
    <Section id="use-cases">
      <Reveal>
        <SectionTitle className="max-w-3xl">
          Built for what needed a <span className="emph">truth layer.</span>
        </SectionTitle>
      </Reveal>

      <RevealGroup className="mt-14 grid gap-5 md:grid-cols-3">
        {CASES.map(({ Icon, title, desc }) => (
          <RevealItem key={title}>
            <div className="card-verdict group h-full p-7 transition-colors hover:border-amber/40">
              <Icon className="h-9 w-9 text-parchment-70 transition-colors group-hover:text-parchment" />
              <h3 className="mt-6 font-display text-2xl text-parchment">{title}</h3>
              <p className="mt-3 font-mono text-[13px] leading-[1.7] text-parchment-55">
                {desc}
              </p>
            </div>
          </RevealItem>
        ))}
      </RevealGroup>
    </Section>
  );
}
