import { Wordmark } from "@/components/icons/Logo";

const LINKS = ["Docs", "GitHub", "Twitter/X", "Discord", "Audit Report", "Terms"];

export function Footer() {
  return (
    <footer className="hairline-t mt-10">
      <div className="mx-auto max-w-6xl px-6 py-14">
        <div className="flex flex-col gap-6 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex items-center gap-4">
            <Wordmark />
            <span className="hidden font-mono text-[12px] text-parchment-40 sm:inline">
              A permissionless truth layer for Solana.
            </span>
          </div>
          <p className="font-ui text-[12px] uppercase tracking-[0.16em] text-parchment-40">
            Built on{" "}
            <span className="bg-gradient-to-r from-[#9945FF] to-[#14F195] bg-clip-text font-bold text-transparent">
              Solana
            </span>
          </p>
        </div>

        <div className="hairline-t mt-10 flex flex-col gap-4 pt-6 sm:flex-row sm:items-center sm:justify-between">
          <ul className="flex flex-wrap gap-x-6 gap-y-2">
            {LINKS.map((l) => (
              <li key={l}>
                <a
                  href="#"
                  className="focus-ring rounded font-mono text-[12px] text-parchment-55 transition-colors hover:text-amber"
                >
                  {l}
                </a>
              </li>
            ))}
          </ul>
          <p className="font-mono text-[11px] text-parchment-40">
            © {new Date().getFullYear()} Cassie Protocol
          </p>
        </div>
      </div>

      {/* Oversized brand wordmark — bleeds to footer edge. */}
      <div className="relative w-full select-none overflow-hidden" aria-hidden>
        <h2 className="font-display whitespace-nowrap text-center font-semibold leading-[0.8] tracking-[0.01em] text-[clamp(5rem,21vw,20rem)] text-transparent [-webkit-text-stroke:1px_var(--color-parchment-15)]">
          CASSIE<span className="text-amber [-webkit-text-stroke:0]">.</span>
        </h2>
        {/* fade base into void */}
        <div className="pointer-events-none absolute inset-x-0 bottom-0 h-1/2 bg-gradient-to-t from-void to-transparent" />
      </div>
    </footer>
  );
}
