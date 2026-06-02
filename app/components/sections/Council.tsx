"use client";

import * as React from "react";
import { motion, animate, useMotionValue, useTransform } from "framer-motion";
import { Section, SectionTitle } from "@/components/ui/section";
import { Reveal } from "@/components/ui/reveal";
import { CountUp } from "@/components/ui/count-up";

const STATS = [
  { to: 9, label: "Active Council Members" },
  { to: 14, label: "Invocations (all time)" },
  { to: 2.1, suffix: "h", decimals: 1, label: "Avg Council Resolution" },
];

export function Council() {
  return (
    <Section id="council">
      <div className="grid items-center gap-14 lg:grid-cols-[1.1fr_0.9fr]">
        <Reveal>
          <SectionTitle className="max-w-xl">
            When the system can&rsquo;t decide, <span className="emph">humans do.</span>
          </SectionTitle>
          <p className="mt-5 font-mono text-[14px] leading-[1.7] text-parchment-70">
            The Council is the last resort, not the first.
          </p>

          <div className="card-verdict mt-9 border-l-2 border-l-amber px-6 py-6">
            <p className="font-mono text-[13.5px] leading-[1.75] text-parchment-70">
              If weighted answers diverge past a configurable threshold — meaning the
              network genuinely can&rsquo;t agree — Cassie escalates to its Council: a set of
              reputation-vetted members who cast a binding vote.
            </p>
            <p className="mt-4 font-mono text-[13.5px] leading-[1.75] text-parchment-70">
              Council seats are earned by protocol reputation, not bought. Members who
              vote with the losing side too often lose their seat. The Council only
              activates when the system escalates — it has no standing authority to
              resolve questions proactively.
            </p>
          </div>

          <p className="mt-6 font-mono text-[12px] text-parchment-40">
            Council is the exception. In 30 days, 0 questions required it.
          </p>
        </Reveal>

        {/* council chamber ring */}
        <Reveal delay={0.1}>
          <CouncilRing />
        </Reveal>
      </div>

      <div className="mt-16 grid gap-px overflow-hidden rounded-md border border-parchment-08 bg-parchment-08 sm:grid-cols-3">
        {STATS.map((s) => (
          <div key={s.label} className="bg-void px-6 py-7 text-center">
            <p className="font-ui text-[30px] font-bold text-amber">
              <CountUp to={s.to} suffix={s.suffix} decimals={s.decimals ?? 0} />
            </p>
            <p className="mt-2 font-mono text-[11px] uppercase tracking-[0.14em] text-parchment-55">
              {s.label}
            </p>
          </div>
        ))}
      </div>
    </Section>
  );
}

const SEATS = 9;
const QUORUM = 5; // majority
const C = 160; // center
const R_SEAT = 118;
const R_ARC = 104;

function CouncilRing() {
  const arcLen = 2 * Math.PI * R_ARC;
  const sealText = "REPUTATION-VETTED · BINDING VOTE · LAST RESORT · ";

  // Rotate via SVG transform attribute about exact center — no bbox drift.
  const angle = useMotionValue(0);
  const spin = useTransform(angle, (a) => `rotate(${a} ${C} ${C})`);
  React.useEffect(() => {
    const controls = animate(angle, 360, {
      duration: 80,
      repeat: Infinity,
      ease: "linear",
    });
    return controls.stop;
  }, [angle]);

  // Quorum arc — draws to majority, holds, retracts, loops.
  const full = arcLen;
  const filled = arcLen * (1 - QUORUM / SEATS);
  const dash = useMotionValue(full);
  React.useEffect(() => {
    const controls = animate(dash, [full, filled, filled, full], {
      duration: 4.5,
      times: [0, 0.35, 0.72, 1],
      repeat: Infinity,
      ease: [0.16, 1, 0.3, 1],
    });
    return controls.stop;
  }, [dash, full, filled]);

  return (
    <div className="relative mx-auto w-full max-w-[360px]">
      <svg viewBox="0 0 320 320" className="w-full" role="img" aria-label="Nine-seat council chamber">
        <defs>
          <path
            id="seal-path"
            d="M160,160 m -140,0 a 140,140 0 1,1 280,0 a 140,140 0 1,1 -280,0"
          />
          <radialGradient id="core-glow">
            <stop offset="0%" stopColor="var(--color-amber)" stopOpacity="0.5" />
            <stop offset="45%" stopColor="var(--color-amber)" stopOpacity="0.18" />
            <stop offset="100%" stopColor="var(--color-amber)" stopOpacity="0" />
          </radialGradient>
        </defs>

        {/* persistent core glow */}
        <circle cx={C} cy={C} r={92} fill="url(#core-glow)" />

        {/* static rings */}
        <circle cx={C} cy={C} r={152} fill="none" stroke="var(--color-parchment-08)" />
        <circle cx={C} cy={C} r={R_ARC + 14} fill="none" stroke="var(--color-parchment-08)" />

        {/* rotating outer ring: ticks + seal text */}
        <motion.g transform={spin}>
          {Array.from({ length: 60 }).map((_, i) => {
            const a = (i / 60) * Math.PI * 2;
            const r1 = 140;
            const r2 = i % 5 === 0 ? 132 : 136;
            return (
              <line
                key={i}
                x1={C + Math.cos(a) * r1}
                y1={C + Math.sin(a) * r1}
                x2={C + Math.cos(a) * r2}
                y2={C + Math.sin(a) * r2}
                stroke="var(--color-parchment-15)"
                strokeWidth={i % 5 === 0 ? 1.4 : 0.8}
              />
            );
          })}
          <text fontFamily="var(--font-mono)" fontSize="8.5" letterSpacing="3" fill="var(--color-amber)" opacity="0.55">
            <textPath href="#seal-path" startOffset="0">
              {sealText.repeat(2)}
            </textPath>
          </text>
        </motion.g>

        {/* quorum arc — fills to majority */}
        <motion.circle
          cx={C}
          cy={C}
          r={R_ARC}
          fill="none"
          stroke="var(--color-amber)"
          strokeWidth={2.5}
          strokeLinecap="round"
          transform={`rotate(-90 ${C} ${C})`}
          strokeDasharray={arcLen}
          style={{ strokeDashoffset: dash }}
        />

        {/* spokes + seats */}
        {Array.from({ length: SEATS }).map((_, i) => {
          const a = (i / SEATS) * Math.PI * 2 - Math.PI / 2;
          const sx = C + Math.cos(a) * R_SEAT;
          const sy = C + Math.sin(a) * R_SEAT;
          const ix = C + Math.cos(a) * 58;
          const iy = C + Math.sin(a) * 58;
          const voted = i < QUORUM;
          return (
            <motion.g
              key={i}
              initial={{ opacity: 0, scale: 0 }}
              whileInView={{ opacity: 1, scale: 1 }}
              viewport={{ once: true }}
              style={{ originX: `${sx}px`, originY: `${sy}px` }}
              transition={{ delay: 0.4 + i * 0.08, type: "spring", stiffness: 320, damping: 15 }}
            >
              <line x1={ix} y1={iy} x2={sx} y2={sy} stroke="var(--color-parchment-08)" strokeWidth={1} />
              <circle
                cx={sx}
                cy={sy}
                r={11}
                fill={voted ? "var(--color-amber)" : "var(--color-void-card)"}
                stroke={voted ? "var(--color-amber)" : "var(--color-parchment-15)"}
                strokeWidth={1.4}
              />
              <text
                x={sx}
                y={sy + 3.5}
                textAnchor="middle"
                fontFamily="var(--font-ui)"
                fontSize="10"
                fontWeight="700"
                fill={voted ? "var(--color-void)" : "var(--color-parchment-55)"}
              >
                {i + 1}
              </text>
            </motion.g>
          );
        })}

        {/* center gavel + label */}
        <g>
          {/* sound block */}
          <rect x={146} y={171} width={28} height={4} rx={1} fill="var(--color-parchment-40)" />
          {/* gavel head */}
          <g transform={`rotate(-32 160 150)`}>
            <rect x={150} y={138} width={20} height={11} rx={2} fill="var(--color-amber)" />
            <rect x={158} y={148} width={4} height={20} rx={1} fill="var(--color-parchment-70)" />
          </g>
        </g>
        <text
          x={C}
          y={200}
          textAnchor="middle"
          fontFamily="var(--font-display)"
          fontSize="20"
          fontWeight="600"
          fill="var(--color-parchment)"
        >
          Council
        </text>
        <text
          x={C}
          y={214}
          textAnchor="middle"
          fontFamily="var(--font-mono)"
          fontSize="7.5"
          letterSpacing="2"
          fill="var(--color-parchment-40)"
        >
          {QUORUM}/{SEATS} QUORUM
        </text>
      </svg>
    </div>
  );
}
