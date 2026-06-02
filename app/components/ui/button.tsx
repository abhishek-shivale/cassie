import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex cursor-pointer items-center justify-center gap-2 whitespace-nowrap rounded-md font-ui font-semibold tracking-wide transition-all duration-200 focus-ring disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50",
  {
    variants: {
      variant: {
        // Amber fill — primary CTA.
        primary:
          "bg-amber text-void hover:bg-amber/90 hover:-translate-y-0.5 shadow-[0_8px_30px_-12px_rgba(245,166,35,0.6)]",
        // Outlined — secondary / ghost.
        ghost:
          "bg-transparent text-parchment border border-parchment-15 hover:border-amber/60 hover:text-amber",
        // Inverted — dark text on amber banners.
        invert:
          "bg-void text-parchment hover:bg-void/80 border border-void",
        // Bare link with amber arrow.
        link: "bg-transparent text-parchment-70 hover:text-amber px-0 h-auto",
      },
      size: {
        sm: "h-9 px-4 text-[12px]",
        md: "h-11 px-5 text-[13px]",
        lg: "h-13 px-7 text-[14px]",
      },
    },
    defaultVariants: { variant: "primary", size: "md" },
  }
);

type Props = React.ButtonHTMLAttributes<HTMLButtonElement> &
  VariantProps<typeof buttonVariants> & { asChild?: boolean };

export const Button = React.forwardRef<HTMLButtonElement, Props>(
  function Button({ className, variant, size, ...props }, ref) {
    return (
      <button
        ref={ref}
        className={cn(buttonVariants({ variant, size }), className)}
        {...props}
      />
    );
  }
);

export { buttonVariants };
