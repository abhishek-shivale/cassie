import { cn } from "@/lib/utils";
import { Reveal } from "./Reveal";

type SectionHeadingProps = {
  eyebrow: string;
  title: React.ReactNode;
  intro?: React.ReactNode;
  align?: "left" | "center";
  className?: string;
};

export function SectionHeading({
  eyebrow,
  title,
  intro,
  align = "left",
  className,
}: SectionHeadingProps) {
  return (
    <div
      className={cn(
        "flex flex-col gap-5",
        align === "center" && "items-center text-center",
        className
      )}
    >
      <Reveal>
        <span className="inline-flex items-center gap-2.5 rounded-full border border-line-strong bg-elevated/60 px-3.5 py-1.5 font-mono text-[11px] uppercase tracking-[0.22em] text-muted">
          {eyebrow}
        </span>
      </Reveal>
      <Reveal delay={0.08}>
        <h2 className="max-w-3xl text-balance text-[clamp(2rem,5vw,3.4rem)] font-semibold text-ink">
          {title}
        </h2>
      </Reveal>
      {intro && (
        <Reveal delay={0.16}>
          <p
            className={cn(
              "max-w-xl text-pretty text-[15px] leading-relaxed text-muted md:text-base",
              align === "center" && "mx-auto"
            )}
          >
            {intro}
          </p>
        </Reveal>
      )}
    </div>
  );
}
