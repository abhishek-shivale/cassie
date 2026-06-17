import { ArrowRight, Terminal } from "lucide-react";
import { Reveal } from "@/components/ui/Reveal";
import { site } from "@/lib/site";

const snippet = [
  { t: "// post a YES/NO question, settle on-chain", c: "text-faint" },
  { t: "await cassie.ask({", c: "text-ink-soft" },
  { t: '  question: "Did event X occur by block N?",', c: "text-ink-soft" },
  { t: "  bounty: 5_000_000,          // lamports", c: "text-ink-soft" },
  { t: "  callbackProgram: myProgram, // optional CPI", c: "text-ink-soft" },
  { t: "});", c: "text-ink-soft" },
  { t: "// → bonded, disputed if wrong, final", c: "text-teal" },
];

export function CTA() {
  return (
    <section id="build" className="relative px-6 py-28 md:py-36">
      <div className="mx-auto max-w-6xl">
        <Reveal>
          <div className="ring-gradient is-active relative overflow-hidden rounded-[2rem] border border-line-strong bg-elevated/50 p-8 md:p-14">
            <div className="pointer-events-none absolute -right-20 -top-24 h-80 w-80 rounded-full bg-[radial-gradient(circle,rgba(139,111,255,0.28),transparent_70%)]" />
            <div className="pointer-events-none absolute -bottom-24 -left-20 h-80 w-80 rounded-full bg-[radial-gradient(circle,rgba(167,139,255,0.2),transparent_70%)]" />

            <div className="relative grid items-center gap-12 lg:grid-cols-[1fr_minmax(0,460px)]">
              <div>
                <span className="inline-flex items-center gap-2.5 rounded-full border border-line-strong bg-base/60 px-3.5 py-1.5 font-mono text-[11px] uppercase tracking-[0.22em] text-muted">
                  For builders
                </span>
                <h2 className="mt-6 text-balance text-[clamp(2rem,5vw,3.2rem)] font-bold text-ink">
                  Build on truth you can{" "}
                  <span className="text-gradient-accent">verify.</span>
                </h2>
                <p className="mt-5 max-w-md text-[15px] leading-relaxed text-ink-soft">
                  Query Cassie from any Solana program or client. Permissionless,
                  composable, and bonded by design — no API keys, no gatekeepers,
                  no trusted middleman.
                </p>
                <div className="mt-9 flex flex-col gap-3 sm:flex-row">
                  <a
                    href={site.links.docs}
                    className="group inline-flex items-center gap-2 rounded-xl bg-ink px-6 py-3.5 text-[15px] font-medium text-void transition-transform hover:-translate-y-0.5"
                  >
                    Start building
                    <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-1" />
                  </a>
                  <a
                    href={site.links.github}
                    className="glass inline-flex items-center gap-2 rounded-xl px-6 py-3.5 text-[15px] font-medium text-ink transition-colors hover:border-line-strong"
                  >
                    View on GitHub
                  </a>
                </div>
              </div>

              <div className="overflow-hidden rounded-2xl border border-line bg-void/80 font-mono text-[13px] shadow-2xl backdrop-blur">
                <div className="flex items-center gap-2 border-b border-line px-4 py-3">
                  <Terminal className="h-3.5 w-3.5 text-muted" />
                  <span className="text-xs text-muted">cassie.ts</span>
                  <span className="ml-auto flex gap-1.5">
                    <span className="h-2.5 w-2.5 rounded-full bg-line-strong" />
                    <span className="h-2.5 w-2.5 rounded-full bg-line-strong" />
                    <span className="h-2.5 w-2.5 rounded-full bg-line-strong" />
                  </span>
                </div>
                <pre className="overflow-x-auto p-5 leading-relaxed">
                  {snippet.map((line, i) => (
                    <div key={i} className={line.c}>
                      <span className="mr-4 select-none text-faint/50">
                        {String(i + 1).padStart(2, "0")}
                      </span>
                      {line.t || " "}
                    </div>
                  ))}
                </pre>
              </div>
            </div>
          </div>
        </Reveal>
      </div>
    </section>
  );
}
