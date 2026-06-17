"use client";

import { ReactLenis } from "lenis/react";
import * as React from "react";

export function SmoothScroll({ children }: { children: React.ReactNode }) {
  return (
    <ReactLenis
      root
      options={{
        lerp: 0.1,
        smoothWheel: true,
        wheelMultiplier: 1,
        touchMultiplier: 1.5,
        anchors: { offset: -72 }, 
      }}
    >
      {children}
    </ReactLenis>
  );
}
