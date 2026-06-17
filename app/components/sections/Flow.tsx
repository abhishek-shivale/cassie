import { SectionHeading } from "@/components/ui/Heading";
import { StickyScroll, type StickyItem } from "@/components/ui/StickyScroll";
import { flow, type FlowStep } from "@/lib/site";
import { cn } from "@/lib/utils";

const accentText = {
  amber: "text-amber",
  teal: "text-teal",
  petrol: "text-petrol",
  copper: "text-copper",
} as const;

const accentDot = {
  amber: "bg-amber shadow-[0_0_18px_3px_rgba(139,111,255,0.7)]",
  teal: "bg-teal shadow-[0_0_18px_3px_rgba(167,139,255,0.7)]",
  petrol: "bg-petrol shadow-[0_0_18px_3px_rgba(109,79,224,0.7)]",
  copper: "bg-copper shadow-[0_0_18px_3px_rgba(91,63,214,0.7)]",
} as const;

const accentGlow = {
  amber:
    "bg-[radial-gradient(ellipse_70%_60%_at_50%_15%,rgba(139,111,255,0.28),transparent_70%)]",
  teal: "bg-[radial-gradient(ellipse_70%_60%_at_50%_15%,rgba(167,139,255,0.24),transparent_70%)]",
  petrol: "bg-[radial-gradient(ellipse_70%_60%_at_50%_15%,rgba(109,79,224,0.26),transparent_70%)]",
  copper:
    "bg-[radial-gradient(ellipse_70%_60%_at_50%_15%,rgba(91,63,214,0.26),transparent_70%)]",
} as const;

function StepVisual({ step }: { step: FlowStep }) {
  return (
    <div className="ring-gradient is-active relative h-full overflow-hidden rounded-3xl border border-line-strong bg-elevated/40 p-8">
      <div className={cn("absolute inset-0", accentGlow[step.accent])} />
      <div className="relative flex h-full flex-col">
        <div className="flex items-center justify-between">
          <span
            className={cn(
              "font-display text-7xl font-bold leading-none",
              accentText[step.accent]
            )}
          >
            {step.index}
          </span>
          <span
            className={cn(
              "h-3 w-3 rounded-full animate-pulse-dot",
              accentDot[step.accent]
            )}
          />
        </div>

        <div className="mt-auto">
          <span className="inline-flex rounded-full border border-line bg-base/60 px-2.5 py-1 font-mono text-[10px] uppercase tracking-[0.18em] text-muted">
            {step.role}
          </span>
          <h4 className="mt-4 font-display text-4xl font-bold text-ink">
            {step.title}
          </h4>
          <p className={cn("mt-2 text-sm font-medium", accentText[step.accent])}>
            {step.blurb}
          </p>
        </div>

        <div className="mt-6 flex items-center gap-1.5">
          {flow.map((s) => (
            <span
              key={s.id}
              className={cn(
                "h-1 flex-1 rounded-full transition-colors",
                s.id === step.id
                  ? accentDot[step.accent].split(" ")[0]
                  : "bg-line-strong"
              )}
            />
          ))}
        </div>
      </div>
    </div>
  );
}

export function Flow() {
  const items: StickyItem[] = flow.map((step) => ({
    eyebrow: `Step ${step.index} · ${step.role}`,
    title: `${step.title} — ${step.blurb}`,
    description: step.detail,
    visual: <StepVisual step={step} />,
  }));

  return (
    <section id="flow" className="relative px-6 py-28 md:py-32">
      <div className="mx-auto max-w-6xl">
        <SectionHeading
          eyebrow="The machine"
          title={
            <>
              How a question becomes{" "}
              <span className="text-gradient-accent">verified truth</span>
            </>
          }
          intro="Five steps, mostly automatic. The optimistic path settles in minutes; escalation only fires when someone bonds against an answer."
        />

        <div className="mt-12">
          <StickyScroll items={items} />
        </div>
      </div>
    </section>
  );
}
