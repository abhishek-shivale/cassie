"use client";

import { motion, useAnimationControls, type Variants } from "framer-motion";
import * as React from "react";

const variants: Variants = {
  hidden: { opacity: 0, y: 20, filter: "blur(6px)" },
  show: {
    opacity: 1,
    y: 0,
    filter: "blur(0px)",
    transition: { duration: 0.7, ease: [0.16, 1, 0.3, 1] },
  },
};

type RevealProps = {
  children: React.ReactNode;
  delay?: number;
  className?: string;
  as?: "div" | "section" | "li" | "span" | "h2" | "p";
};

export function Reveal({ children, delay = 0, className, as = "div" }: RevealProps) {
  const MotionTag = motion[as] as typeof motion.div;
  const ref = React.useRef<HTMLElement | null>(null);
  const controls = useAnimationControls();
  const shown = React.useRef(false);

  React.useEffect(() => {
    const el = ref.current;
    const reveal = () => {
      if (shown.current) return;
      shown.current = true;
      controls.start("show");
    };

    let io: IntersectionObserver | undefined;
    if (el && "IntersectionObserver" in window) {
      io = new IntersectionObserver(
        (entries) => entries.forEach((e) => e.isIntersecting && reveal()),
        { rootMargin: "0px 0px -8% 0px" }
      );
      io.observe(el);
    }
    const t = window.setTimeout(reveal, 1400);

    return () => {
      io?.disconnect();
      window.clearTimeout(t);
    };
  }, [controls]);

  return (
    <MotionTag
      ref={ref as never}
      className={className}
      variants={variants}
      initial="hidden"
      animate={controls}
      transition={{ delay }}
    >
      {children}
    </MotionTag>
  );
}
