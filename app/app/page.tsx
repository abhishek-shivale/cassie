import { Nav } from "@/components/sections/Nav";
import { Hero } from "@/components/sections/Hero";
import { Flow } from "@/components/sections/Flow";
import { CTA } from "@/components/sections/CTA";
import { Footer } from "@/components/sections/Footer";
import { Chrome } from "@/components/ui/Chrome";

export default function Page() {
  return (
    <main className="relative">
      <Nav />
      <Hero />
      <Flow />
      <CTA />
      <Footer />
    </main>
  );
}
