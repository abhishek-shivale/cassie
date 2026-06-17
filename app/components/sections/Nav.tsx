"use client";

import { useEffect, useState } from "react";
import { Logo } from "@/components/ui/Logo";
import { nav, site } from "@/lib/site";
import { cn } from "@/lib/utils";

export function Nav() {
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 24);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <header className="fixed inset-x-0 top-0 z-50 flex justify-center px-4 pt-4">
      <nav
        className={cn(
          "flex w-full max-w-6xl items-center justify-between rounded-2xl px-4 py-2.5 transition-all duration-500 md:px-5",
          scrolled
            ? "glass border-line-strong shadow-[0_20px_60px_-30px_rgba(0,0,0,0.9)]"
            : "border border-transparent"
        )}
      >
        <a href="#top" aria-label="Cassie home">
          <Logo />
        </a>

        <div className="hidden items-center gap-1 md:flex">
          {nav.map((item) => (
            <a
              key={item.href}
              href={item.href}
              className="rounded-lg px-3.5 py-2 text-sm text-ink-soft/80 transition-colors hover:text-ink"
            >
              {item.label}
            </a>
          ))}
        </div>

        <div className="flex items-center gap-2">
          <a
            href={site.links.docs}
            className="hidden rounded-lg px-3.5 py-2 text-sm text-ink-soft/80 transition-colors hover:text-ink sm:block"
          >
            Docs
          </a>
          <a
            href={site.links.app}
            className="ring-gradient is-active group relative rounded-lg bg-ink px-4 py-2 text-sm font-medium text-void transition-transform hover:-translate-y-0.5"
          >
            Launch app
          </a>
        </div>
      </nav>
    </header>
  );
}
