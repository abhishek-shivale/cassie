import { SectionHeading } from "@/components/ui/Heading";
import { Reveal } from "@/components/ui/Reveal";
import { councilPoints } from "@/lib/site";

function EscalationDiagram() {
  return (
    <div className="relative mx-auto aspect-square w-full max-w-md">
      <div className="absolute inset-0 rounded-full bg-[radial-gradient(circle_at_center,rgba(139,111,255,0.18),transparent_65%)]" />

      <div className="absolute inset-0 flex items-start justify-center rounded-full border border-line">
        <span className="-mt-3 rounded-full border border-line-strong bg-base px-3 py-1 font-mono text-[10px] uppercase tracking-[0.2em] text-teal">
          Council
        </span>
      </div>

      <div
        className="absolute inset-[14%] flex items-start justify-center rounded-full border border-amber/25"
        style={{ animation: "orbit-spin 60s linear infinite" }}
      >
        <span
          className="-mt-3 rounded-full border border-amber/30 bg-base px-3 py-1 font-mono text-[10px] uppercase tracking-[0.2em] text-amber"
          style={{ animation: "orbit-spin 60s linear infinite reverse" }}
        >
          Reputation vote
        </span>
      </div>

      <div className="absolute inset-[30%] flex flex-col items-center justify-center rounded-full border border-teal/30 bg-teal/5 text-center backdrop-blur-sm">
        <span className="font-display text-3xl font-bold text-ink">99%</span>
        <span className="mt-1 px-4 font-mono text-[10px] uppercase tracking-[0.16em] text-teal">
          settle here
        </span>
      </div>
    </div>
  );
}

export function Council() {
  return (
    <section id="council" className="relative px-6 py-28 md:py-36">
      <div className="bg-grid pointer-events-none absolute inset-0 opacity-30" />
      <div className="relative mx-auto grid max-w-6xl items-center gap-14 lg:grid-cols-[1fr_minmax(0,440px)]">
        <div>
          <SectionHeading
            eyebrow="The backstop"
            title={
              <>
                A council that fires{" "}
                <span className="text-gradient-accent">only when truth is contested</span>
              </>
            }
            intro="Most oracles centralize trust by default. Cassie inverts it: decentralized resolution is the norm, and human judgment is the rare, bounded, accountable exception."
          />

          <div className="mt-12 flex flex-col gap-px overflow-hidden rounded-2xl border border-line bg-line">
            {councilPoints.map((p, i) => (
              <Reveal key={p.title} delay={i * 0.08}>
                <div className="bg-elevated/40 p-6 transition-colors hover:bg-elevated/70 md:p-7">
                  <h3 className="font-display text-lg font-semibold text-ink">
                    {p.title}
                  </h3>
                  <p className="mt-2 text-sm leading-relaxed text-muted">
                    {p.body}
                  </p>
                </div>
              </Reveal>
            ))}
          </div>
        </div>

        <Reveal delay={0.15} className="animate-float">
          <EscalationDiagram />
        </Reveal>
      </div>
    </section>
  );
}
