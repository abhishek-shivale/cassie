import { cn } from "@/lib/utils";

export function Wordmark({ className }: { className?: string }) {
  return (
    <span
      className={cn(
        "font-display text-[22px] font-semibold leading-none tracking-[0.01em] text-parchment",
        className
      )}
    >
      CASSIE<span className="brand-dot" aria-hidden />
    </span>
  );
}

/** Concentric verdict seal with circular text. */
export function Seal({ className, size = 96 }: { className?: string; size?: number }) {
  const r = size / 2;
  const text = "· VERIFIED ONCHAIN · CASSIE ";
  return (
    <svg
      width={size}
      height={size}
      viewBox={`0 0 ${size} ${size}`}
      className={className}
      aria-hidden
    >
      <defs>
        <path
          id="seal-arc"
          d={`M ${r},${r} m -${r - 9},0 a ${r - 9},${r - 9} 0 1,1 ${(r - 9) * 2},0 a ${r - 9},${r - 9} 0 1,1 -${(r - 9) * 2},0`}
        />
      </defs>
      <circle cx={r} cy={r} r={r - 1} fill="none" stroke="var(--color-amber)" strokeWidth="1" opacity="0.5" />
      <circle cx={r} cy={r} r={r - 6} fill="none" stroke="var(--color-amber)" strokeWidth="0.5" opacity="0.3" />
      <text fill="var(--color-amber)" fontFamily="var(--font-mono), monospace" fontSize="6.5" letterSpacing="1.6" opacity="0.75">
        <textPath href="#seal-arc" startOffset="0">
          {text.repeat(3)}
        </textPath>
      </text>
      <circle cx={r} cy={r} r={3.5} fill="var(--color-amber)" />
    </svg>
  );
}
