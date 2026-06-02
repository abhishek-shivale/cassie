import * as React from "react";
import { cn } from "@/lib/utils";

type Props = React.HTMLAttributes<HTMLSpanElement> & {
  tone?: "default" | "proposed" | "settled" | "disputed";
  dot?: boolean;
};

// Status pills. Brand: sage = settled, amber-tint = proposed, ink = disputed.
export function Badge({ className, tone = "default", dot = true, children, ...props }: Props) {
  const tones = {
    default:
      "text-ink/55 border-ink/15 bg-paper",
    proposed:
      "text-amber-deep border-amber/40 bg-amber-tint",
    settled:
      "text-sage border-sage/40 bg-sage/10",
    disputed:
      "text-ink border-ink/20 bg-paper",
  } as const;

  const dotColor = {
    default: "bg-ink/40",
    proposed: "bg-amber",
    settled: "bg-sage",
    disputed: "bg-ink",
  } as const;

  return (
    <span
      className={cn(
        "inline-flex items-center gap-2 rounded-full border px-3 py-1 font-mono text-[11px] tracking-[0.18em] uppercase",
        tones[tone],
        className
      )}
      {...props}
    >
      {dot && <span className={cn("size-1.5 rounded-full", dotColor[tone])} aria-hidden />}
      {children}
    </span>
  );
}
