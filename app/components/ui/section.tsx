import { cn } from "@/lib/utils";
import * as React from "react";

export function Section({
  id,
  className,
  children,
}: {
  id?: string;
  className?: string;
  children: React.ReactNode;
}) {
  return (
    <section
      id={id}
      className={cn("relative mx-auto w-full max-w-6xl px-6 py-24 sm:py-32", className)}
    >
      {children}
    </section>
  );
}

export function Eyebrow({ children }: { children: React.ReactNode }) {
  return <p className="eyebrow mb-5">{children}</p>;
}

export function SectionTitle({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <h2
      className={cn(
        "font-display text-4xl leading-[1.06] tracking-[-0.01em] sm:text-5xl",
        className
      )}
    >
      {children}
    </h2>
  );
}
