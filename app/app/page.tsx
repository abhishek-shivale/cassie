import { Nav } from "@/components/sections/Nav";
import { Hero } from "@/components/sections/Hero";
// import { Ticker } from "@/components/sections/Ticker";
import { Problem } from "@/components/sections/Problem";
import { HowItWorks } from "@/components/sections/HowItWorks";
import { Honesty } from "@/components/sections/Honesty";
import { Reputation } from "@/components/sections/Reputation";
import { Council } from "@/components/sections/Council";
import { UseCases } from "@/components/sections/UseCases";
import { Architecture } from "@/components/sections/Architecture";
import { DeveloperCTA } from "@/components/sections/DeveloperCTA";
import { Footer } from "@/components/sections/Footer";

export default function Page() {
  return (
    <>
      <Nav />
      <main>
        <Hero />
        {/* <Ticker /> */}
        <Problem />
        <HowItWorks />
        <Honesty />
        <Reputation />
        <Council />
        <UseCases />
        <Architecture />
        <DeveloperCTA />
      </main>
      <Footer />
    </>
  );
}
