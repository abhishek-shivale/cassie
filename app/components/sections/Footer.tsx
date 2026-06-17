import { Logo } from "@/components/ui/Logo";
import { nav, site } from "@/lib/site";

const groups = [
  {
    title: "Protocol",
    links: [
      { label: "How it works", href: "#flow" },
      { label: "Mechanics", href: "#mechanics" },
      { label: "Council", href: "#council" },
    ],
  },
  {
    title: "Developers",
    links: [
      { label: "Documentation", href: site.links.docs },
      { label: "GitHub", href: site.links.github },
      { label: "Launch app", href: site.links.app },
    ],
  },
  {
    title: "Community",
    links: [
      { label: "X / Twitter", href: site.links.x },
      { label: "Discord", href: "#" },
      { label: "Governance", href: "#" },
    ],
  },
];

export function Footer() {
  return (
    <footer className="relative border-t border-line px-6 pb-12 pt-16">
      <div className="mx-auto max-w-6xl">
        <div className="grid gap-12 md:grid-cols-[1.4fr_repeat(3,1fr)]">
          <div>
            <Logo />
            <p className="mt-4 max-w-xs text-sm leading-relaxed text-muted">
              A permissionless optimistic oracle on Solana. Engineered to be
              trusted with money.
            </p>
            <div className="mt-5 inline-flex items-center gap-2 rounded-full border border-line bg-elevated/40 px-3 py-1.5 font-mono text-[11px] text-muted">
              All systems operational
            </div>
          </div>

          {groups.map((g) => (
            <div key={g.title}>
              <h4 className="font-mono text-[11px] uppercase tracking-[0.2em] text-faint">
                {g.title}
              </h4>
              <ul className="mt-4 flex flex-col gap-3">
                {g.links.map((l) => (
                  <li key={l.label}>
                    <a
                      href={l.href}
                      className="text-sm text-ink-soft/75 transition-colors hover:text-ink"
                    >
                      {l.label}
                    </a>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        <div className="mt-14 flex flex-col items-start justify-between gap-4 border-t border-line pt-8 sm:flex-row sm:items-center">
          <p className="font-mono text-xs text-faint">
            © {new Date().getFullYear()} {site.name}. Built on Solana.
          </p>
          <nav className="flex gap-5">
            {nav.map((n) => (
              <a
                key={n.href}
                href={n.href}
                className="font-mono text-xs text-faint transition-colors hover:text-muted"
              >
                {n.label}
              </a>
            ))}
          </nav>
        </div>
      </div>
    </footer>
  );
}
