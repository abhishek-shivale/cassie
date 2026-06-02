"use client";

import * as React from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Button } from "@/components/ui/button";
import { CountUp } from "@/components/ui/count-up";

const QUESTION = "Did Solana TPS exceed 65,000 on block #312,847,001?";

const STATS = [
  { to: 2841, label: "Questions Settled" },
  { to: 1.2, prefix: "$", suffix: "M", decimals: 1, label: "Bounties Paid" },
  { to: 99.3, suffix: "%", decimals: 1, label: "Resolved Optimistically" },
  { to: 0, label: "Council Overrules (30d)" },
];

function useTyped(text: string, start: boolean, speed = 38) {
  const [out, setOut] = React.useState("");
  React.useEffect(() => {
    if (!start) return;
    let i = 0;
    const id = setInterval(() => {
      i += 1;
      setOut(text.slice(0, i));
      if (i >= text.length) clearInterval(id);
    }, speed);
    return () => clearInterval(id);
  }, [text, start, speed]);
  return out;
}

function QuestionCard() {
  const [phase, setPhase] = React.useState<"typing" | "settled">("typing");
  const typed = useTyped(QUESTION, true);
  const done = typed.length >= QUESTION.length;

  React.useEffect(() => {
    if (done) {
      const id = setTimeout(() => setPhase("settled"), 900);
      return () => clearTimeout(id);
    }
  }, [done]);

  return (
    <div className="relative">
      <div className="card-verdict relative overflow-hidden p-0 shadow-[0_40px_120px_-40px_rgba(0,0,0,0.8)]">
        {/* header row */}
        <div className="hairline-b flex items-center justify-between px-6 py-4">
          <span className={`pill ${phase === "settled" ? "pill--settled" : "pill--open"}`}>
            {phase === "settled" ? "Settled" : "Open Question"}
          </span>
          <span className="font-mono text-[12px] text-parchment-55">
            BOUNTY: <span className="text-amber">50◎</span>
          </span>
        </div>

        {/* question body */}
        <div className="px-6 py-8">
          <p className="font-display min-h-[88px] text-[26px] leading-[1.25] text-parchment caret sm:text-[30px]">
            <span className="text-parchment-40">&ldquo;</span>
            {typed}
            <span className="text-parchment-40">{done ? "”" : ""}</span>
          </p>
        </div>

        {/* meta */}
        <div className="hairline-t grid grid-cols-3 divide-x divide-parchment-08 font-mono text-[11px]">
          <Meta k="Window" v={phase === "settled" ? "Closed" : "4h 22m"} />
          <Meta k="Answers" v="3" />
          <Meta k="Bonds" v="180◎" />
        </div>

        {/* actions */}
        <div className="hairline-t flex items-center gap-3 px-6 py-4">
          <Button size="sm" variant="ghost" className="flex-1">
            Answer This
          </Button>
          <Button size="sm" variant="ghost" className="flex-1">
            View Details
          </Button>
        </div>

        {/* scan line shimmer while typing */}
        {!done && (
          <motion.div
            aria-hidden
            className="pointer-events-none absolute inset-x-0 top-0 h-px bg-amber/50"
            animate={{ y: [0, 320, 0] }}
            transition={{ duration: 2.4, repeat: Infinity, ease: "easeInOut" }}
          />
        )}
      </div>

      {/* SETTLED stamp */}
      <AnimatePresence>
        {phase === "settled" && (
          <motion.div
            className="pointer-events-none absolute -right-3 -top-4 z-10"
            initial={{ opacity: 0, scale: 1.8, rotate: 24 }}
            animate={{ opacity: 1, scale: 1, rotate: 11 }}
            transition={{ type: "spring", stiffness: 320, damping: 14 }}
          >
            <span className="stamp stamp--settled text-[15px]">Settled ✓</span>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

function Meta({ k, v }: { k: string; v: string }) {
  return (
    <div className="px-4 py-3">
      <p className="text-parchment-40">{k}</p>
      <p className="mt-0.5 text-parchment">{v}</p>
    </div>
  );
}

export function Hero() {
  return (
    <section className="relative overflow-hidden px-6 pb-24 pt-36 sm:pt-44">
      {/* amber bloom behind card */}
      <div
        aria-hidden
        className="pointer-events-none absolute right-[6%] top-[18%] -z-10 h-[420px] w-[420px] rounded-full opacity-20 blur-[120px]"
        style={{ background: "var(--color-amber)" }}
      />

      <div className="mx-auto grid max-w-6xl items-center gap-16 lg:grid-cols-[1.05fr_0.95fr]">
        {/* left copy */}
        <div>
          <h1 className="font-display text-[clamp(2.6rem,6vw,4.6rem)] font-medium leading-[1.04] tracking-[-0.015em]">
            Solana has price feeds.
            <br />
            It didn&rsquo;t have <span className="emph">truth.</span>
            <br />
            Until now.
          </h1>

          <p className="mt-7 max-w-xl font-mono text-[14px] leading-[1.7] text-parchment-70">
            Cassie is a permissionless optimistic oracle for arbitrary truth claims on
            Solana. Did it happen? Cassie decides — onchain, without permission, without
            bridges.
          </p>

          <div className="mt-9 flex flex-wrap gap-3">
            <Button size="lg">Post a Question →</Button>
            <Button size="lg" variant="ghost">
              Read the Docs
            </Button>
          </div>
        </div>

        {/* right card */}
        <div>
          <QuestionCard />
        </div>
      </div>

      {/* live stats */}
      <div className="mx-auto mt-20 grid max-w-6xl grid-cols-2 gap-px overflow-hidden rounded-md border border-parchment-08 bg-parchment-08 lg:grid-cols-4">
        {STATS.map((s) => (
          <div key={s.label} className="bg-void px-6 py-7">
            <p className="font-ui text-[30px] font-bold leading-none tracking-tight text-amber sm:text-[34px]">
              <CountUp
                to={s.to}
                prefix={s.prefix}
                suffix={s.suffix}
                decimals={s.decimals ?? 0}
              />
            </p>
            <p className="mt-3 font-mono text-[11px] uppercase tracking-[0.14em] text-parchment-55">
              {s.label}
            </p>
          </div>
        ))}
      </div>
    </section>
  );
}
