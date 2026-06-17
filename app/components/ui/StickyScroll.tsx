"use client";

import { motion, useMotionValueEvent, useScroll } from "framer-motion";
import { useRef, useState } from "react";

export type StickyItem = {
  eyebrow?: string;
  title: string;
  description: string;
  visual: React.ReactNode;
};

export function StickyScroll({ items }: { items: StickyItem[] }) {
  const [active, setActive] = useState(0);
  const ref = useRef<HTMLDivElement>(null);
  const { scrollYProgress } = useScroll({
    target: ref,
    offset: ["start 28%", "end 72%"],
  });

  useMotionValueEvent(scrollYProgress, "change", (latest) => {
    const idx = Math.min(
      items.length - 1,
      Math.max(0, Math.floor(latest * items.length))
    );
    setActive(idx);
  });

  return (
    <div ref={ref} className="relative grid gap-10 lg:grid-cols-2 lg:gap-16">
      <div>
        {items.map((item, i) => (
          <div
            key={i}
            className="flex min-h-[58vh] flex-col justify-center py-8 lg:min-h-[72vh]"
          >
            <div className="mb-7 lg:hidden">{item.visual}</div>

            <motion.div
              animate={{
                opacity: active === i ? 1 : 0.32,
                filter: active === i ? "blur(0px)" : "blur(1.5px)",
              }}
              transition={{ duration: 0.4, ease: "easeOut" }}
            >
              {item.eyebrow && (
                <span className="font-mono text-xs uppercase tracking-[0.2em] text-muted">
                  {item.eyebrow}
                </span>
              )}
              <h3 className="mt-3 font-display text-3xl font-semibold text-ink md:text-4xl">
                {item.title}
              </h3>
              <p className="mt-4 max-w-md text-[15px] leading-relaxed text-muted">
                {item.description}
              </p>
            </motion.div>
          </div>
        ))}
      </div>

      <div className="hidden lg:block">
        <div className="sticky top-28 h-[26rem] w-full">
          {items.map((item, i) => (
            <motion.div
              key={i}
              animate={{
                opacity: active === i ? 1 : 0,
                scale: active === i ? 1 : 0.94,
                y: active === i ? 0 : 12,
              }}
              transition={{ duration: 0.45, ease: [0.16, 1, 0.3, 1] }}
              style={{ pointerEvents: active === i ? "auto" : "none" }}
              className="absolute inset-0"
            >
              {item.visual}
            </motion.div>
          ))}
        </div>
      </div>
    </div>
  );
}
