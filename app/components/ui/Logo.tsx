import { cn } from "@/lib/utils";

export function Logo({ className }: { className?: string }) {
  return (
    <span
      className={cn(
        "font-display text-[25px] font-bold tracking-tight text-amber",
        className,
      )}
    >
      Cassie
    </span>
  );
}
