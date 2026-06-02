// Flat ink line-art with a single amber dot accent. No gradients, no fills beyond palette.

type Props = { className?: string };

const STROKE = "currentColor";
const DOT = "var(--color-amber)";

export function GradDefs() {
  return null;
}

export function MarketsIcon({ className = "" }: Props) {
  return (
    <svg viewBox="0 0 40 40" className={className} fill="none" aria-hidden>
      <path d="M5 30 L13 22 L19 26 L27 14 L35 18" stroke={STROKE} strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round" />
      <circle cx="27" cy="14" r="2.4" fill={DOT} />
      <line x1="5" y1="34" x2="35" y2="34" stroke={STROKE} strokeOpacity="0.25" strokeWidth="1" />
    </svg>
  );
}

export function InsuranceIcon({ className = "" }: Props) {
  return (
    <svg viewBox="0 0 40 40" className={className} fill="none" aria-hidden>
      <path d="M20 5 L33 11 L33 21 C33 28 27 33 20 35 C13 33 7 28 7 21 L7 11 Z" stroke={STROKE} strokeWidth="1.4" strokeLinejoin="round" />
      <path d="M14 20 L18 24 L26 16" stroke={STROKE} strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round" />
      <circle cx="26" cy="16" r="1.6" fill={DOT} />
    </svg>
  );
}

export function BridgeIcon({ className = "" }: Props) {
  return (
    <svg viewBox="0 0 40 40" className={className} fill="none" aria-hidden>
      <circle cx="9" cy="20" r="5" stroke={STROKE} strokeWidth="1.4" />
      <circle cx="31" cy="20" r="5" stroke={STROKE} strokeWidth="1.4" />
      <path d="M14 20 L26 20" stroke={STROKE} strokeWidth="1.4" strokeLinecap="round" />
      <circle cx="20" cy="20" r="2.2" fill={DOT} />
    </svg>
  );
}

export function RWAIcon({ className = "" }: Props) {
  return (
    <svg viewBox="0 0 40 40" className={className} fill="none" aria-hidden>
      <rect x="8" y="6" width="20" height="26" rx="1" stroke={STROKE} strokeWidth="1.4" />
      <path d="M22 6 L28 12 L22 12 Z" stroke={STROKE} strokeWidth="1.2" />
      <line x1="12" y1="16" x2="22" y2="16" stroke={STROKE} strokeWidth="1.2" strokeLinecap="round" />
      <line x1="12" y1="20" x2="24" y2="20" stroke={STROKE} strokeWidth="1.2" strokeLinecap="round" />
      <line x1="12" y1="24" x2="20" y2="24" stroke={STROKE} strokeWidth="1.2" strokeLinecap="round" />
      <circle cx="29" cy="29" r="2.4" fill={DOT} />
    </svg>
  );
}

export function BountyIcon({ className = "" }: Props) {
  return (
    <svg viewBox="0 0 40 40" className={className} fill="none" aria-hidden>
      <path d="M20 4 L26 10 L36 10 L29 17 L32 26 L20 21 L8 26 L11 17 L4 10 L14 10 Z" stroke={STROKE} strokeWidth="1.4" strokeLinejoin="round" />
      <circle cx="20" cy="14" r="2" fill={DOT} />
    </svg>
  );
}

export function MilestoneIcon({ className = "" }: Props) {
  return (
    <svg viewBox="0 0 40 40" className={className} fill="none" aria-hidden>
      <line x1="5" y1="20" x2="35" y2="20" stroke={STROKE} strokeWidth="1.4" strokeLinecap="round" />
      <circle cx="10" cy="20" r="2.4" stroke={STROKE} strokeWidth="1.4" />
      <circle cx="20" cy="20" r="2.4" stroke={STROKE} strokeWidth="1.4" />
      <circle cx="30" cy="20" r="3.2" fill={DOT} />
      <path d="M30 12 L30 8 M27 11 L30 8 L33 11" stroke={STROKE} strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

export function ElectionsIcon({ className = "" }: Props) {
  return (
    <svg viewBox="0 0 40 40" className={className} fill="none" aria-hidden>
      <rect x="6" y="14" width="28" height="20" rx="1" stroke={STROKE} strokeWidth="1.4" />
      <path d="M14 14 L14 8 L26 8 L26 14" stroke={STROKE} strokeWidth="1.4" strokeLinejoin="round" />
      <line x1="13" y1="22" x2="27" y2="22" stroke={STROKE} strokeWidth="1.2" strokeLinecap="round" />
      <line x1="13" y1="28" x2="22" y2="28" stroke={STROKE} strokeWidth="1.2" strokeLinecap="round" />
    </svg>
  );
}

export function SportsIcon({ className = "" }: Props) {
  return (
    <svg viewBox="0 0 40 40" className={className} fill="none" aria-hidden>
      <circle cx="20" cy="20" r="14" stroke={STROKE} strokeWidth="1.4" />
      <path d="M20 6 L20 34 M6 20 L34 20" stroke={STROKE} strokeWidth="1" strokeOpacity="0.7" />
      <circle cx="20" cy="20" r="3" fill={DOT} />
    </svg>
  );
}
