import { Check } from "lucide-react";
import { SectionHeading } from "@/components/ui/Heading";
import { Reveal } from "@/components/ui/Reveal";
import { mechanics } from "@/lib/site";
import { cn } from "@/lib/utils";

const glow = {
  amber:
    "before:bg-[radial-gradient(ellipse_60%_50%_at_30%_0%,rgba(139,111,255,0.2),transparent_70%)]",
  teal: "before:bg-[radial-gradient(ellipse_60%_50%_at_70%_0%,rgba(167,139,255,0.18),transparent_70%)]",
} as const;

const kbdColor = {
  amber: "text-amber border-amber/30 bg-amber/10",
  teal: "text-teal border-teal/30 bg-teal/10",
} as const;

const tick = {
  amber: "text-amber",
  teal: "text-teal",
} as const;

export function Mechanics() {
  return (
    <section id="mechanics" className="relative px-6 py-28 md:py-36">
      <div className="mx-auto max-w-6xl">
        <SectionHeading
          eyebrow="Why it holds"
          title={
            <>
              Economics and reputation,
              <br className="hidden md:block" /> doing the work of trust
            </>
          }
          intro="Cassie does not ask you to trust a name. It makes honesty the rational, profitable move — and lets accuracy compound into influence over time."
        />

        <div className="mt-16 grid gap-5 md:grid-cols-2">
          {mechanics.map((m, i) => (
            <Reveal key={m.id} delay={i * 0.08}>
              <article
                className={cn(
                  "ring-gradient group relative h-full overflow-hidden rounded-3xl border border-line bg-elevated/40 p-8 transition-colors hover:border-line-strong md:p-10",
                  "before:absolute before:inset-0 before:opacity-0 before:transition-opacity before:duration-500 before:content-[''] group-hover:before:opacity-100",
                  glow[m.accent]
                )}
              >
                <div className="relative">
                  <span
                    className={cn(
                      "inline-flex rounded-md border px-2 py-1 font-mono text-[10px] font-medium tracking-[0.18em]",
                      kbdColor[m.accent]
                    )}
                  >
                    {m.kbd}
                  </span>
                  <h3 className="mt-5 font-display text-2xl font-semibold text-ink md:text-[28px]">
                    {m.title}
                  </h3>
                  <p className="mt-3 text-[15px] leading-relaxed text-muted">
                    {m.body}
                  </p>
                  <ul className="mt-6 flex flex-col gap-3 border-t border-line pt-6">
                    {m.points.map((p) => (
                      <li
                        key={p}
                        className="flex items-start gap-3 text-sm text-ink-soft"
                      >
                        <Check
                          className={cn("mt-0.5 h-4 w-4 shrink-0", tick[m.accent])}
                        />
                        {p}
                      </li>
                    ))}
                  </ul>
                </div>
              </article>
            </Reveal>
          ))}
        </div>
      </div>
    </section>
  );
}
