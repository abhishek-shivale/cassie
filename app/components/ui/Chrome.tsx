"use client";

import { useEffect, useState } from "react";
import { nav } from "@/lib/site";
import { cn } from "@/lib/utils";

export function Chrome() {
  const [progress, setProgress] = useState(0);
  const [active, setActive] = useState(0);

  useEffect(() => {
    const sections = ["top", ...nav.map((n) => n.href.slice(1))];
    const onScroll = () => {
      const max = document.documentElement.scrollHeight - window.innerHeight;
      setProgress(max > 0 ? window.scrollY / max : 0);
      const mid = window.scrollY + window.innerHeight * 0.4;
      let idx = 0;
      sections.forEach((id, i) => {
        const el = document.getElementById(id);
        if (el && el.offsetTop <= mid) idx = i;
      });
      setActive(idx);
    };
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  const labels = ["TOP", ...nav.map((n) => n.label.toUpperCase())];

  return (
    <>
      <div
        aria-hidden
        className="pointer-events-none fixed inset-0 z-[60] opacity-[0.04] mix-blend-overlay"
        style={{
          backgroundImage:
            "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='200' height='200'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='3'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E\")",
        }}
      />

      <div aria-hidden className="pointer-events-none fixed inset-0 z-[55] hidden md:block">
        {(
          [
            "left-5 top-5 border-l border-t",
            "right-5 top-5 border-r border-t",
            "left-5 bottom-5 border-l border-b",
            "right-5 bottom-5 border-r border-b",
          ] as const
        ).map((pos) => (
          <span
            key={pos}
            className={cn("absolute h-5 w-5 border-line-strong", pos)}
          />
        ))}
      </div>

      <div
        aria-hidden
        className="pointer-events-none fixed right-6 top-1/2 z-[55] hidden -translate-y-1/2 flex-col items-end gap-3 lg:flex"
      >
        {labels.map((label, i) => (
          <div key={label} className="flex items-center gap-2.5">
            <span
              className={cn(
                "font-mono text-[9px] tracking-[0.2em] transition-all duration-300",
                i === active ? "text-amber opacity-100" : "text-faint opacity-0"
              )}
            >
              {label}
            </span>
            <span
              className={cn(
                "h-px transition-all duration-300",
                i === active ? "w-6 bg-amber" : "w-3 bg-line-strong"
              )}
            />
          </div>
        ))}
        <span className="mt-1 font-mono text-[9px] tabular-nums text-faint">
          {String(Math.round(progress * 100)).padStart(3, "0")}
        </span>
      </div>
    </>
  );
}
