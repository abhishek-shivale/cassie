import type { Metadata } from "next";
import { Cormorant_Garamond, IBM_Plex_Mono, Syne } from "next/font/google";
import { SmoothScroll } from "@/components/SmoothScroll";
import "./globals.css";

const cormorant = Cormorant_Garamond({
  variable: "--font-display",
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
  style: ["normal", "italic"],
  display: "swap",
});

const plexMono = IBM_Plex_Mono({
  variable: "--font-mono",
  subsets: ["latin"],
  weight: ["400", "500", "600"],
  display: "swap",
});

const syne = Syne({
  variable: "--font-ui",
  subsets: ["latin"],
  weight: ["500", "600", "700", "800"],
  display: "swap",
});

export const metadata: Metadata = {
  title: "Cassie° — A truth layer for Solana",
  description:
    "Cassie is a permissionless optimistic oracle for arbitrary truth claims on Solana. Did it happen? Cassie decides — onchain, without permission, without bridges.",
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html
      lang="en"
      className={`${cormorant.variable} ${plexMono.variable} ${syne.variable}`}
      style={{ colorScheme: "dark" }}
    >
      <head>
        <meta name="color-scheme" content="dark only" />
      </head>
      <body className="min-h-screen bg-void text-parchment antialiased">
        <SmoothScroll>{children}</SmoothScroll>
      </body>
    </html>
  );
}
