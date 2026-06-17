import type { Metadata } from "next";
import { Bricolage_Grotesque, Geist, JetBrains_Mono } from "next/font/google";
import { SmoothScroll } from "@/components/SmoothScroll";
import "./globals.css";

const bricolage = Bricolage_Grotesque({
  subsets: ["latin"],
  variable: "--font-bricolage",
  weight: ["400", "500", "600", "700", "800"],
  display: "swap",
});

const geist = Geist({
  subsets: ["latin"],
  variable: "--font-geist",
  display: "swap",
});

const mono = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-mono",
  weight: ["400", "500", "600"],
  display: "swap",
});

export const metadata: Metadata = {
  title: "Cassie — Permissionless Optimistic Oracle on Solana",
  description:
    "Cassie is a permissionless optimistic oracle on Solana. Anyone posts a question with a bounty, anyone answers by bonding tokens, and disputes resolve through reputation-weighted voting — escalating to a trusted council only when truth is genuinely contested.",
  metadataBase: new URL("https://cassie.xyz"),
  openGraph: {
    title: "Cassie — Permissionless Optimistic Oracle on Solana",
    description:
      "Settled truth, on Solana. Post a question with a bounty, bond an answer, dispute through reputation-weighted voting.",
    type: "website",
  },
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html
      lang="en"
      className={`${bricolage.variable} ${geist.variable} ${mono.variable}`}
      style={{ colorScheme: "dark" }}
    >
      <head>
        <meta name="color-scheme" content="dark only" />
      </head>
      <body>
        <SmoothScroll>{children}</SmoothScroll>
      </body>
    </html>
  );
}
