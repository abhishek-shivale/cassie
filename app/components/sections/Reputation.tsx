"use client";

import { motion } from "framer-motion";
import { Section, SectionTitle } from "@/components/ui/section";
import { Reveal, RevealGroup, RevealItem } from "@/components/ui/reveal";
import { CountUp } from "@/components/ui/count-up";

const TIERS = [
  { name: "New Participant", weight: 1, won: 0, accuracy: "—", barTo: 12 },
  { name: "Established Answerer", weight: 3.4, won: 12, accuracy: "91%", barTo: 42 },
  { name: "Veteran", weight: 8.7, won: 47, accuracy: "97.3%", barTo: 100, top: true },
];

export function Reputation() {
  return (
    <Section id="reputation">
      <Reveal>
        <SectionTitle className="max-w-4xl">
          Skin in the game gets you in.
          <br />
          Your record gives you <span className="emph">power.</span>
        </SectionTitle>
      </Reveal>

      <RevealGroup className="mt-14 grid gap-5 md:grid-cols-3">
        {TIERS.map((t) => (
          <RevealItem key={t.name}>
            <div
              className={
                "card-verdict relative h-full overflow-hidden p-6 " +
                (t.top ? "border-amber/40" : "")
              }
            >
              {t.top && (
                <span className="stamp stamp--new absolute right-4 top-4 rotate-6 text-[10px]">
                  Top Tier
                </span>
              )}
              <p className="font-ui text-[13px] font-bold uppercase tracking-[0.12em] text-parchment-55">
                {t.name}
              </p>

              <p className="mt-5 font-display text-[56px] leading-none text-amber">
                <CountUp to={t.weight} decimals={t.weight % 1 ? 1 : 0} suffix="×" />
              </p>
              <p className="mt-1 font-mono text-[11px] uppercase tracking-[0.14em] text-parchment-40">
                Aggregation Weight
              </p>

              {/* weight bar */}
              <div className="mt-6 h-1.5 w-full overflow-hidden rounded-full bg-parchment-08">
                <motion.div
                  className="h-full rounded-full bg-amber"
                  initial={{ width: 0 }}
                  whileInView={{ width: `${t.barTo}%` }}
                  viewport={{ once: true }}
                  transition={{ duration: 1.4, ease: [0.16, 1, 0.3, 1] }}
                />
              </div>

              <dl className="mt-6 space-y-2 font-mono text-[12.5px]">
                <Row k="Bond required" v="Standard" />
                <Row k="Disputes won" v={String(t.won)} />
                <Row k="Accuracy" v={t.accuracy} />
              </dl>
            </div>
          </RevealItem>
        ))}
      </RevealGroup>

      <Reveal delay={0.05}>
        <div className="mt-12 grid gap-8 lg:grid-cols-2">
          <p className="font-mono text-[14px] leading-[1.75] text-parchment-70">
            Every answer you submit is tracked — not just whether you were right, but how
            calibrated you were. Answerers who consistently post correct, well-timed
            answers earn higher weight in aggregation.
          </p>
          <p className="font-mono text-[14px] leading-[1.75] text-parchment-70">
            This isn&rsquo;t a whitelist. A fresh wallet and a veteran stake the same bond.
            But in a disputed vote, the veteran&rsquo;s position counts more — making
            long-term manipulation more expensive than it&rsquo;s worth.
          </p>
        </div>
      </Reveal>
    </Section>
  );
}

function Row({ k, v }: { k: string; v: string }) {
  return (
    <div className="hairline-t flex items-center justify-between pt-2 text-parchment-55">
      <dt>{k}</dt>
      <dd className="text-parchment">{v}</dd>
    </div>
  );
}
