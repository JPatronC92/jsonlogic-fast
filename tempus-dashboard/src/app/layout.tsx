import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Tempus | Deterministic Billing",
  description: "High-performance time-travel pricing engine.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={`${geistSans.variable} ${geistMono.variable}`}>
        <nav style={{ padding: '1rem 2rem', borderBottom: '1px solid #334155', display: 'flex', gap: '2rem', background: '#0f172a' }}>
          <div style={{ fontWeight: 'bold', color: '#60a5fa' }}>TEMPUS</div>
          <a href="/" style={{ color: '#cbd5e1', textDecoration: 'none' }}>Simulator</a>
          <a href="/builder" style={{ color: '#cbd5e1', textDecoration: 'none' }}>Visual Rule Builder</a>
        </nav>
        {children}
      </body>
    </html>
  );
}
