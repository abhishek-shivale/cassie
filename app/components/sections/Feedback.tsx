import { MessageSquare } from "lucide-react";
import { Reveal } from "@/components/ui/Reveal";

const FORM_URL =
  "https://docs.google.com/forms/d/e/1FAIpQLSeWT61YgmoY4plD44nkY-RhkxYQ7UXctNFCwlmfyqeh8ODIxg/viewform?embedded=true";

const FORM_LINK =
  "https://docs.google.com/forms/d/e/1FAIpQLSeWT61YgmoY4plD44nkY-RhkxYQ7UXctNFCwlmfyqeh8ODIxg/viewform";

export function Feedback() {
  return (
    <section id="feedback" className="relative px-6 py-24 md:py-32">
      <div className="mx-auto max-w-6xl">
        <Reveal>
          <div className="mb-12 text-center">
            <span className="inline-flex items-center gap-2.5 rounded-full border border-line-strong bg-base/60 px-3.5 py-1.5 font-mono text-[11px] uppercase tracking-[0.22em] text-muted">
              <MessageSquare className="h-3 w-3" />
              Shape what we build
            </span>
            <h2 className="mt-6 text-balance text-[clamp(1.8rem,4vw,2.8rem)] font-bold text-ink">
              Tell us what you&apos;d{" "}
              <span className="text-gradient-accent">build with it.</span>
            </h2>
            <p className="mx-auto mt-4 max-w-md text-[15px] leading-relaxed text-ink-soft">
              On-chain truth, bonded by design. Your answers shape what Cassie
              becomes. Takes 2 minutes.
            </p>
          </div>
        </Reveal>

        <Reveal>
          <div className="relative overflow-hidden rounded-[2rem] border border-line-strong bg-elevated/30 shadow-2xl">
            <div className="pointer-events-none absolute -right-24 -top-24 h-72 w-72 rounded-full bg-[radial-gradient(circle,rgba(139,111,255,0.18),transparent_70%)]" />
            <div className="pointer-events-none absolute -bottom-24 -left-24 h-72 w-72 rounded-full bg-[radial-gradient(circle,rgba(167,139,255,0.12),transparent_70%)]" />

            <div className="relative">
              <iframe
                src={FORM_URL}
                title="Cassie feedback form"
                width="100%"
                height="700"
                className="block w-full border-0"
                loading="lazy"
              />
            </div>

            <div className="border-t border-line px-8 py-4 text-center">
              <a
                href={FORM_LINK}
                target="_blank"
                rel="noopener noreferrer"
                className="font-mono text-[11px] text-muted transition-colors hover:text-ink"
              >
                Open in new tab ↗
              </a>
            </div>
          </div>
        </Reveal>
      </div>
    </section>
  );
}
