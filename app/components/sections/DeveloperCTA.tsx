import { Button } from "@/components/ui/button";

export function DeveloperCTA() {
  return (
    <section id="docs" className="px-6 py-20">
      <div className="card-verdict relative mx-auto max-w-6xl overflow-hidden rounded-lg border-amber/25 px-8 py-16 text-parchment sm:px-14 sm:py-20">
        {/* soft amber wash, top-right */}
        <div
          aria-hidden
          className="pointer-events-none absolute -right-20 -top-24 h-72 w-72 rounded-full opacity-15 blur-[110px]"
          style={{ background: "var(--color-amber)" }}
        />
        <div className="relative max-w-2xl">
          <h2 className="font-display text-[clamp(2.2rem,5vw,3.6rem)] font-semibold leading-[1.05] tracking-[-0.015em]">
            Ask anything. <span className="emph">Trust the answer.</span>
          </h2>
          <p className="mt-5 font-mono text-[14px] leading-[1.7] text-parchment-70">
            Cassie&rsquo;s SDK drops in 3 lines of code. Docs, examples, and a devnet
            deployment are ready.
          </p>
          <div className="mt-9 flex flex-wrap gap-3">
            <Button size="lg">Read the Docs →</Button>
            <Button variant="ghost" size="lg">
              View on GitHub
            </Button>
            <Button variant="ghost" size="lg">
              Join Discord
            </Button>
          </div>
        </div>
      </div>
    </section>
  );
}
