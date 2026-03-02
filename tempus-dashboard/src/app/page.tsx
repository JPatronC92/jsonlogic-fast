"use client";

import Link from "next/link";
import styles from "./landing.module.css";

export default function LandingPage() {
    return (
        <main className={styles.container}>
            {/* Hero Section */}
            <section className={styles.hero}>
                <div className={styles.glowBlob}></div>
                <h1 className={styles.headline}>
                    Stop the <span className={styles.highlight}>$100M</span> Billing Drift.
                </h1>
                <p className={styles.subheadline}>
                    Tempus is the deterministic pricing infrastructure for high-scale Fintechs.
                    Guarantee sub-millisecond precision, absolute auditability, and zero floating-point errors.
                </p>
                <div className={styles.ctaGroup}>
                    <Link href="/dashboard" className={styles.primaryBtn}>
                        Launch Simulator
                    </Link>
                    <Link href="/builder" className={styles.secondaryBtn}>
                        Explore Rule Builder
                    </Link>
                </div>

                <div className={styles.statsRow}>
                    <div className={styles.statBox}>
                        <h3>1.3M</h3>
                        <p>TPS (Rust Core)</p>
                    </div>
                    <div className={styles.statBox}>
                        <h3>100%</h3>
                        <p>Deterministic Output</p>
                    </div>
                    <div className={styles.statBox}>
                        <h3>0</h3>
                        <p>Historical Overlaps</p>
                    </div>
                </div>
            </section>

            {/* Features Section */}
            <section className={styles.features}>
                <h2>Why CFOs and CTOs choose Tempus</h2>
                <div className={styles.featureGrid}>
                    <div className={styles.featureCard}>
                        <div className={styles.iconWrapper}>🕰️</div>
                        <h3>Absolute Time-Travel Audit</h3>
                        <p>Leveraging PostgreSQL DATERANGE, Tempus mathematically guarantees that pricing versions never overlap. Replay any historical transaction precisely.</p>
                    </div>
                    <div className={styles.featureCard}>
                        <div className={styles.iconWrapper}>🦀</div>
                        <h3>Rust-Powered Speed</h3>
                        <p>Evaluating logic at high-frequency trading speed (1,349,995 evaluations per second via PyO3). Ready for Stripe-scale workloads.</p>
                    </div>
                    <div className={styles.featureCard}>
                        <div className={styles.iconWrapper}>🧠</div>
                        <h3>Zero-Float Determinism</h3>
                        <p>Using strict decimal handling and json-logic, we ensure identical outputs for identical inputs—forever. No more reconciliation nightmares.</p>
                    </div>
                    <div className={styles.featureCard}>
                        <div className={styles.iconWrapper}>🏢</div>
                        <h3>Multi-Tenant Security</h3>
                        <p>Secure JWT and API Key isolation natively built-in for B2B/SaaS scale operations. Complete financial data segregation.</p>
                    </div>
                </div>
            </section>

            {/* Code / Visual Demo Snippet */}
            <section className={styles.technicalShowcase}>
                <div className={styles.codeWindow}>
                    <div className={styles.windowHeader}>
                        <span className={styles.dotRed}></span>
                        <span className={styles.dotYellow}></span>
                        <span className={styles.dotGreen}></span>
                        <span className={styles.windowTitle}>tempus_core valuation</span>
                    </div>
                    <pre>
                        {`[Tempus Compiler] Translating visual scheme to Rust Core...
[Rule: Marketplace MX] -> Compiled successfully.
Executing 50,000 transactions (Batch ID: #8b3a9)...
✓ Processed 50,000 txs in 0.037s (1.35M TPS)
[Success] Zero drift detected. Audit logs committed.`}
                    </pre>
                </div>
            </section>
        </main>
    );
}
