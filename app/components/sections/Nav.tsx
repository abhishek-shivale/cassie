"use client";

import * as React from "react";
import { Wordmark } from "@/components/icons/Logo";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

const links = [
  { label: "How It Works", href: "#how" },
  { label: "Reputation", href: "#reputation" },
  { label: "Council", href: "#council" },
  { label: "Use Cases", href: "#use-cases" },
  { label: "Docs", href: "#docs" },
  { label: "GitHub", href: "#github" },
];

export function Nav() {
  const [scrolled, setScrolled] = React.useState(false);
  const [open, setOpen] = React.useState(false);

  React.useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 12);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <header
      className={cn(
        "fixed inset-x-0 top-0 z-50 transition-all duration-300",
        scrolled ? "frost hairline-b" : "bg-transparent"
      )}
    >
      <nav className="mx-auto flex h-16 max-w-6xl items-center justify-between px-6">
        <a href="#" aria-label="Cassie home" className="focus-ring rounded">
          <Wordmark />
        </a>

        <ul className="hidden items-center gap-7 lg:flex">
          {links.map((l) => (
            <li key={l.label}>
              <a
                href={l.href}
                className="focus-ring rounded font-mono text-[12px] tracking-[0.04em] text-parchment-70 transition-colors hover:text-amber"
              >
                {l.label}
              </a>
            </li>
          ))}
        </ul>

        <div className="hidden lg:block">
          <Button size="sm">Post a Question →</Button>
        </div>

        <button
          className="focus-ring cursor-pointer rounded p-2 lg:hidden"
          aria-label="Toggle menu"
          aria-expanded={open}
          onClick={() => setOpen((v) => !v)}
        >
          <div className="space-y-[5px]">
            <span className={cn("block h-px w-6 bg-parchment transition", open && "translate-y-[6px] rotate-45")} />
            <span className={cn("block h-px w-6 bg-parchment transition", open && "opacity-0")} />
            <span className={cn("block h-px w-6 bg-parchment transition", open && "-translate-y-[6px] -rotate-45")} />
          </div>
        </button>
      </nav>

      {open && (
        <div className="frost hairline-t lg:hidden">
          <ul className="mx-auto flex max-w-6xl flex-col gap-1 px-6 py-4">
            {links.map((l) => (
              <li key={l.label}>
                <a
                  href={l.href}
                  onClick={() => setOpen(false)}
                  className="block py-2 font-mono text-sm text-parchment-70 hover:text-amber"
                >
                  {l.label}
                </a>
              </li>
            ))}
            <li className="pt-2">
              <Button size="sm" className="w-full">
                Post a Question →
              </Button>
            </li>
          </ul>
        </div>
      )}
    </header>
  );
}
