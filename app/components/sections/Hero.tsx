"use client";

import { motion } from "framer-motion";
import { ArrowRight, Copy, Check } from "lucide-react";
import { useState } from "react";
import { CountUp } from "@/components/ui/CountUp";
import { site, stats } from "@/lib/site";

const ease = [0.16, 1, 0.3, 1] as const;

export function Hero() {
  const [copied, setCopied] = useState(false);

  const copyProgram = async () => {
    try {
      await navigator.clipboard.writeText(site.links.program);
      setCopied(true);
      setTimeout(() => setCopied(false), 1600);
    } catch {
      // ignore
    }
  };

  return (
    <section
      id="top"
      className="relative flex min-h-screen flex-col items-center justify-center overflow-hidden px-6 pt-32 pb-20"
    >
      <div className="pointer-events-none absolute inset-0 -z-10 overflow-hidden">
        <div className="aurora" />
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_75%_65%_at_50%_38%,transparent,rgba(8,8,12,0.55)_72%,var(--color-void)_100%)]" />
        <div className="absolute inset-0 bg-[linear-gradient(180deg,rgba(8,8,12,0.55)_0%,transparent_28%,transparent_52%,var(--color-void)_100%)]" />
      </div>

      <div className="bg-grid pointer-events-none absolute inset-0 -z-10 opacity-40" />

      <div className="relative z-10 flex w-full max-w-5xl flex-col items-center text-center">
        <motion.a
          href={site.links.app}
          initial={{ opacity: 0, y: 16 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.7, ease }}
          className="ring-gradient group mb-8 inline-flex items-center gap-2.5 rounded-full border border-line-strong bg-elevated/50 py-1.5 pl-2.5 pr-3.5 font-mono text-[11px] uppercase tracking-[0.18em] text-ink-soft backdrop-blur-md"
        >
          <span className="flex items-center gap-2 rounded-full bg-teal/10 px-2 py-0.5 text-teal">
            Live
          </span>
          Devnet-alpha on Solana
          <ArrowRight className="h-3 w-3 transition-transform group-hover:translate-x-0.5" />
        </motion.a>

        <motion.h1
          initial={{ opacity: 0, y: 24, filter: "blur(10px)" }}
          animate={{ opacity: 1, y: 0, filter: "blur(0px)" }}
          transition={{ duration: 0.9, ease, delay: 0.08 }}
          className="text-balance text-[clamp(2.6rem,8vw,6rem)] font-bold leading-[0.95] text-ink"
        >
          Answers you can
          <br />
          <span className="relative whitespace-nowrap text-amber">
            bet on
            <span
              aria-hidden
              className="absolute -bottom-1 left-0 h-px w-full bg-gradient-to-r from-gold via-amber to-transparent shadow-[0_0_12px_rgba(139,111,255,0.7)]"
            />
          </span>
          <span className="text-amber">.</span>
        </motion.h1>

        <motion.p
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, ease, delay: 0.22 }}
          className="mt-7 max-w-2xl text-pretty text-base leading-relaxed text-ink-soft/85 md:text-lg"
        >
          Ask a yes-or-no question with a bounty. Others bond their answer,
          and anyone can challenge it. Reputation + stake decides what&rsquo;s true — no
          oracle to trust, no gatekeeper.
        </motion.p>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, ease, delay: 0.34 }}
          className="mt-10 flex flex-col items-center gap-3 sm:flex-row"
        >
          <a
            href={site.links.app}
            className="ring-gradient is-active group inline-flex items-center gap-2 rounded-xl bg-ink px-6 py-3.5 text-[15px] font-medium text-void transition-transform hover:-translate-y-0.5"
          >
            Query the oracle
            <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-1" />
          </a>
          <a
            href={site.links.docs}
            className="glass inline-flex items-center gap-2 rounded-xl px-6 py-3.5 text-[15px] font-medium text-ink transition-colors hover:border-line-strong"
          >
            Read the docs
          </a>
        </motion.div>

        <motion.button
          onClick={copyProgram}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.8, ease, delay: 0.5 }}
          className="group mt-7 inline-flex items-center gap-2.5 rounded-lg border border-line bg-base/40 px-3.5 py-2 font-mono text-xs text-faint backdrop-blur transition-colors hover:border-line-strong hover:text-muted"
        >
          <span className="text-amber">program</span>
          <span className="truncate max-w-[200px] sm:max-w-none">
            {site.links.program}
          </span>
          {copied ? (
            <Check className="h-3.5 w-3.5 text-teal" />
          ) : (
            <Copy className="h-3.5 w-3.5 opacity-60 group-hover:opacity-100" />
          )}
        </motion.button>
      </div>

      <motion.div
        initial={{ opacity: 0, y: 28 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.9, ease, delay: 0.64 }}
        className="relative z-10 mt-16 grid w-full max-w-5xl grid-cols-2 gap-px overflow-hidden rounded-2xl border border-line bg-line md:grid-cols-4"
      >
        {stats.map((s) => (
          <div
            key={s.label}
            className="flex flex-col gap-1.5 bg-base/70 px-5 py-6 backdrop-blur-sm"
          >
            <span className="font-display text-2xl font-semibold text-ink md:text-3xl">
              <CountUp
                to={s.value}
                decimals={"decimals" in s ? s.decimals : 0}
                prefix={"prefix" in s ? s.prefix : ""}
                suffix={s.suffix}
              />
            </span>
            <span className="text-xs text-muted">{s.label}</span>
          </div>
        ))}
      </motion.div>
    </section>
  );
}
