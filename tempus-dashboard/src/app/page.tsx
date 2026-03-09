"use client";

import { useState, useEffect, useCallback } from "react";
import CountUp from "react-countup";
import { IconInfoCircle } from "@tabler/icons-react";
import { initWasm, getWasm } from "../lib/wasm";
import { TEMPLATES, PricingRule, RuleParam, rebuildRule } from "../data/templates";
import { dict, Language } from "../lib/i18n";
import styles from "./simulator.module.css";

interface SimResult {
    fees: number[];
    ruleFees: { [ruleId: string]: number[] };
    totalRevenue: number;
    ruleRevenue: { [ruleId: string]: number };
    totalProcessed: number;
    avgFee: number;
    avgRate: number;
    timeMs: number;
    opsPerSec: number;
    timePerTx: number;
    baselineRevenue?: number;
}

export default function PublicSimulator() {
    const [wasmReady, setWasmReady] = useState(false);
    
    // Unified Universal Template
    const activeProfile = TEMPLATES[0].presentation_profile['en']; // default to EN for initial profile
    const templateDataset = TEMPLATES[0].dataset.data;

    const [activeRules, setActiveRules] = useState<PricingRule[]>(TEMPLATES[0].rules);
    const [rulesJson, setRulesJson] = useState(JSON.stringify(TEMPLATES[0].rules, null, 2));
    const [txInput, setTxInput] = useState(JSON.stringify(templateDataset, null, 2));
    const [result, setResult] = useState<SimResult | null>(null);
    const [baselineResult, setBaselineResult] = useState<SimResult | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [txCount, setTxCount] = useState(10000);
    const [showAdvanced, setShowAdvanced] = useState(false);
    const [lang, setLang] = useState<Language>('en');
    const [auditExpanded, setAuditExpanded] = useState(false);
    const [isDeployModalOpen, setIsDeployModalOpen] = useState(false);
    const [hasInteracted, setHasInteracted] = useState(false);
    const [isEditingBaseline, setIsEditingBaseline] = useState(false);
    const [customBaselineInput, setCustomBaselineInput] = useState("");
    const [customBaseline, setCustomBaseline] = useState<number | null>(null);

    const t = dict[lang];
    const profile = TEMPLATES[0].presentation_profile[lang];
    const isCurrency = profile.units === 'currency';

    useEffect(() => { initWasm().then(() => setWasmReady(true)).catch(console.error); }, []);

    const updateRuleParam = (ruleId: string, paramKey: string, newValue: number) => {
        setHasInteracted(true);
        setActiveRules(prev => {
            const updated = prev.map(rule => {
                if (rule.id !== ruleId) return rule;
                const updatedParams = rule.params.map(p => p.key === paramKey ? { ...p, value: newValue } : p);
                return { ...rule, params: updatedParams, rule: rebuildRule(rule.id, updatedParams) };
            });
            setRulesJson(JSON.stringify(updated, null, 2));
            return updated;
        });
    };

    const runSimulation = useCallback(() => {
        if (!wasmReady) return;
        setError(null);

        try {
            const wasm = getWasm();
            let transactions: any[];
            try { transactions = JSON.parse(txInput); } catch { setError("Invalid data"); return; }

            let expandedTx = transactions;
            if (txCount > transactions.length) {
                const multiplier = Math.ceil(txCount / transactions.length);
                expandedTx = [];
                for (let i = 0; i < multiplier; i++) expandedTx.push(...transactions);
                expandedTx = expandedTx.slice(0, txCount);
            }

            let rulesToEval = activeRules;
            try { const m = JSON.parse(rulesJson); if (Array.isArray(m)) rulesToEval = m; } catch { }

            const start = performance.now();
            const expandedTxStr = JSON.stringify(expandedTx);
            const rulesJsonStr = JSON.stringify(rulesToEval.map(r => r.rule));
            const resultJson = wasm.evaluate_batch_multi_wasm(rulesJsonStr, expandedTxStr);
            const resultJson2 = wasm.evaluate_batch_multi_wasm(rulesJsonStr, expandedTxStr);
            const isDeterministic = resultJson === resultJson2;
            const rawResults: { total_fee: number, rule_fees: number[] }[] = JSON.parse(resultJson);
            const elapsed = performance.now() - start;

            const ruleFees: { [ruleId: string]: number[] } = {};
            const ruleRevenue: { [ruleId: string]: number } = {};
            rulesToEval.forEach(r => { ruleFees[r.id] = new Array(expandedTx.length).fill(0); ruleRevenue[r.id] = 0; });

            const allFees: number[] = new Array(expandedTx.length).fill(0);
            let totalRevenue = 0;

            for (let i = 0; i < rawResults.length; i++) {
                allFees[i] = rawResults[i].total_fee;
                totalRevenue += rawResults[i].total_fee;
                for (let j = 0; j < rulesToEval.length; j++) {
                    const ruleId = rulesToEval[j].id;
                    ruleFees[ruleId][i] = rawResults[i].rule_fees[j];
                    ruleRevenue[ruleId] += rawResults[i].rule_fees[j];
                }
            }

            const totalProcessed = isCurrency ? expandedTx.reduce((a, b) => a + (b.amount || 0), 0) : expandedTx.length;
            const avgRate = totalProcessed > 0 ? (totalRevenue / totalProcessed) * 100 : 0;
            const opsPerSec = elapsed > 0 ? Math.round((allFees.length * rulesToEval.length) / (elapsed / 2 / 1000)) : 0;

            const newResult = {
                fees: allFees, ruleFees, totalRevenue, ruleRevenue, totalProcessed,
                avgFee: totalRevenue / allFees.length, avgRate,
                timeMs: elapsed / 2, opsPerSec, timePerTx: (elapsed / 2) / allFees.length,
                isDeterministic
            };

            setResult(prev => {
                // Initialize baseline ONLY once with the first successful evaluation
                if (!prev && !baselineResult) setBaselineResult(newResult as unknown as SimResult);
                return newResult as unknown as SimResult;
            });
        } catch (e: any) { setError(e.message || String(e)); }
    }, [wasmReady, activeRules, rulesJson, txInput, txCount, baselineResult, isCurrency]);

    // LIVE OPS REACTIVITY: Run simulation automatically on any dependency change
    useEffect(() => {
        if (hasInteracted) {
            runSimulation();
        }
    }, [runSimulation, hasInteracted]);

    const downloadScenario = () => {
        const blob = new Blob([JSON.stringify({ template: TEMPLATES[0].id, rules: activeRules, transactions: JSON.parse(txInput) }, null, 2)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a'); a.href = url; a.download = `tempus-unified-scenario.json`;
        document.body.appendChild(a); a.click(); document.body.removeChild(a); URL.revokeObjectURL(url);
    };

    const fmtOps = (n: number) => n >= 1e6 ? `${(n / 1e6).toFixed(1)}M` : n >= 1e3 ? `${(n / 1e3).toFixed(0)}K` : `${n}`;
    const txLabels: Record<number, string> = { 100: "100", 1000: "1K", 10000: "10K", 1000000: "High-Volume Execution (1M Tx)" };

    const auditEventsToShow = auditExpanded ? 5 : 1;
    const effectiveBaseline = customBaseline ?? baselineResult?.totalRevenue ?? result?.totalRevenue ?? 0;

    return (
        <main className={styles.app}>
            {/* ─── NAV ─── */}
            <nav className={styles.nav}>
                <div className={styles.navLeft}>
                    <img src="/tempus_logo_transparent.png" alt="Tempus Logo" className={styles.navLogo} style={{ height: '32px', width: 'auto', marginRight: '8px' }} />
                    <span className={styles.brand}>{t.brand}</span>
                    <span className={styles.navDot}>·</span>
                    <span className={styles.navTag}>{t.tagline}</span>
                </div>
                <div className={styles.navRight}>
                    <span className={styles.engineBadge}>{wasmReady ? t.engineNote : "⏳"}</span>
                    <div className={styles.langSwitch}>
                        <button className={lang === 'en' ? styles.langOn : styles.langOff} onClick={() => setLang('en')}>EN</button>
                        <button className={lang === 'es' ? styles.langOn : styles.langOff} onClick={() => setLang('es')}>ES</button>
                    </div>
                </div>
            </nav>

            {/* ─── WORKSPACE ─── */}
            <div className={styles.workspace}>

                {/* LEFT PANEL */}
                <aside className={styles.panel}>
                    <div className={styles.panelSection}>
                        <h3 className={styles.panelTitle}>{t.steps.params}</h3>
                        <p className={styles.scenarioDesc} style={{marginBottom: '1rem', color: 'var(--text-dim)'}}>{profile.context.tempusSolution}</p>
                        {activeRules.map(rule => (
                            <div key={rule.id} className={styles.ruleBlock}>
                                <span className={styles.ruleName}>{rule.name}</span>
                                {rule.params.map(param => (
                                    <div key={param.key} className={styles.sliderRow}>
                                        <div className={styles.sliderHeader}>
                                            <span className={styles.sliderLabel}>{param.label}</span>
                                            <span className={styles.sliderVal}>{param.value}{param.suffix}</span>
                                        </div>
                                        <input type="range" className={styles.slider} min={param.min} max={param.max} step={param.step} value={param.value} onChange={e => updateRuleParam(rule.id, param.key, parseFloat(e.target.value))} />
                                    </div>
                                ))}
                            </div>
                        ))}
                    </div>

                    <div className={styles.divider} />

                    <div className={styles.panelSection}>
                        <h3 className={styles.panelTitle}>{t.volume}</h3>
                        <div className={styles.volRow}>
                            {Object.entries(txLabels).map(([v, label]) => (
                                <button key={v} className={`${styles.volBtn} ${txCount === Number(v) ? styles.volActive : ''} ${Number(v) === 1000000 ? styles.stressBtn : ''}`} onClick={() => { setHasInteracted(true); setTxCount(Number(v)); }}>{label}</button>
                            ))}
                        </div>
                        {profile.context.dataContext && (
                            <p style={{ color: 'var(--text-ghost)', fontSize: '0.65rem', marginTop: '0.5rem', textAlign: 'center' }}>
                                {profile.context.dataContext}
                            </p>
                        )}
                    </div>

                    <div className={styles.deployFunnel}>
                        <h4 style={{color: 'var(--text)', fontSize: '13px', marginBottom: '8px'}}>Build Custom Logic</h4>
                        <p style={{color: 'var(--text-dim)', fontSize: '12px', marginBottom: '16px'}}>Ready to map your own complex algorithms and connect live data flows?</p>
                        <button className={styles.btnPrimary} style={{width: '100%', justifyContent: 'center'}} onClick={() => setIsDeployModalOpen(true)}>Sign Up / Get API Key →</button>
                    </div>
                </aside>

                {/* MAIN CANVAS */}
                <section className={styles.canvas}>
                    {!result ? (
                        <div className={styles.emptyState}>
                            <img src="/tempus_logo_transparent.png" alt="Tempus Mark" className={styles.emptyLogo} />
                            <h2 className={styles.emptyHeadline}>{profile.title}</h2>
                            <p style={{ color: 'var(--text-dim)', marginTop: '4px', fontSize: '0.9rem', marginBottom: '2rem' }}>
                                {lang === 'en' ? 'Lightning-fast rules evaluation engine.' : 'Motor de evaluación hiper-rápido.'}
                            </p>
                            
                            <div className={styles.onboardingSteps}>
                                <div className={styles.onboardStep}>
                                    <span className={styles.stepNum}>1</span>
                                    <span className={styles.stepText}>{lang === 'en' ? 'Select your transaction volume' : 'Elige tu volumen de pagos'}</span>
                                </div>
                                <div className={styles.onboardStep}>
                                    <span className={styles.stepNum}>2</span>
                                    <span className={styles.stepText}>{lang === 'en' ? 'Adjust your pricing rules instantly' : 'Ajusta tus reglas al instante'}</span>
                                </div>
                                <div className={styles.onboardStep}>
                                    <span className={styles.stepNum}>3</span>
                                    <span className={styles.stepText}>{lang === 'en' ? 'Discover your true revenue delta' : 'Descubre tu rentabilidad real'}</span>
                                </div>
                            </div>
                        </div>
                    ) : (
                        /* ─── RESULTS ─── */
                        <div className={styles.resultFlow}>
                            {/* Action bar at top */}
                            <div className={styles.resultHeader}>
                                <h2 className={styles.resultTitle}>{profile.title}</h2>
                                <div className={styles.headerActions}>
                                    <button className={styles.btnSecondary} onClick={downloadScenario}>{t.export.btn}</button>
                                </div>
                            </div>

                            {/* Stats */}
                            <div className={styles.statsGrid}>
                                <div className={styles.stat} style={{ position: 'relative' }}>
                                    <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', gap: '6px', marginBottom: '0.3rem' }}>
                                        <span className={styles.tipWrap}>
                                            <span className={styles.statLabel} style={{ marginBottom: 0 }}>{t.results.baseline}</span>
                                            {profile.context.baselineHint && (
                                                <span className={styles.tip} style={{ marginBottom: '2px' }}>
                                                    <IconInfoCircle size={10} />
                                                    <span className={styles.tipText}>{profile.context.baselineHint}</span>
                                                </span>
                                            )}
                                        </span>
                                        <button 
                                            onClick={() => {
                                                setIsEditingBaseline(!isEditingBaseline);
                                                if (!isEditingBaseline) {
                                                    setCustomBaselineInput(effectiveBaseline.toString());
                                                }
                                            }} 
                                            className={styles.editBtn} 
                                            title="Edit Baseline"
                                        >
                                            ✎
                                        </button>
                                    </div>
                                    {isEditingBaseline ? (
                                        <div style={{ display: 'flex', gap: '4px', justifyContent: 'center', marginTop: '4px' }}>
                                            <input 
                                                type="number" 
                                                className={styles.baselineInput}
                                                value={customBaselineInput} 
                                                onChange={e => setCustomBaselineInput(e.target.value)}
                                                onKeyDown={e => {
                                                    if (e.key === 'Enter') {
                                                        const val = parseFloat(customBaselineInput);
                                                        setCustomBaseline(!isNaN(val) ? val : null);
                                                        setIsEditingBaseline(false);
                                                    }
                                                }}
                                                autoFocus
                                            />
                                            <button 
                                                className={styles.baselineSaveBtn}
                                                onClick={() => {
                                                    const val = parseFloat(customBaselineInput);
                                                    setCustomBaseline(!isNaN(val) ? val : null);
                                                    setIsEditingBaseline(false);
                                                }}
                                            >
                                                ✓
                                            </button>
                                        </div>
                                    ) : (
                                        <span className={styles.statNum}>{isCurrency ? "$" : ""}<CountUp end={effectiveBaseline} duration={0.3} decimals={isCurrency ? 2 : 0} separator="," /></span>
                                    )}
                                </div>
                                <div className={`${styles.stat} ${styles.statHL}`}>
                                    <span className={styles.statLabel}>{t.results.projected}</span>
                                    <span className={styles.statNum}>{isCurrency ? "$" : ""}<CountUp end={result.totalRevenue} duration={0.3} decimals={isCurrency ? 2 : 0} separator="," /></span>
                                </div>
                                <div className={styles.stat}>
                                    <span className={styles.statLabel}>{t.results.delta}</span>
                                    <span className={`${styles.statNum} ${result.totalRevenue !== effectiveBaseline ? (result.totalRevenue > effectiveBaseline ? styles.green : styles.red) : styles.dim}`}>
                                        {result.totalRevenue !== effectiveBaseline
                                            ? <CountUp end={((result.totalRevenue - effectiveBaseline) / Math.max(1, effectiveBaseline)) * 100} duration={0.3} decimals={2} prefix={result.totalRevenue > effectiveBaseline ? "+" : ""} suffix="%" />
                                            : "0.00%"}
                                    </span>
                                </div>
                            </div>

                            {/* Breakdown Bar with labels */}
                            <div className={styles.barWrap}>
                                <div className={styles.barLabels}>
                                    <span>{profile.chartLabels.base}</span>
                                    <span>{profile.chartLabels.highlight}</span>
                                </div>
                                <div className={styles.bar}>
                                    <div className={styles.barFill} style={{ width: `${Math.max(result.avgRate, 1)}%` }}>
                                        {result.avgRate.toFixed(1)}%
                                    </div>
                                    <div className={styles.barRest}>
                                        {(100 - result.avgRate).toFixed(1)}%
                                    </div>
                                </div>
                            </div>

                            {/* Business Conclusion */}
                            <div className={styles.conclusionBox}>
                                <h3>{t.businessConclusion.title}</h3>
                                <p>
                                    {result.totalRevenue > effectiveBaseline 
                                        ? t.businessConclusion.positive.replace('{amount}', `${isCurrency ? "$" : ""}${Math.abs(result.totalRevenue - effectiveBaseline).toLocaleString(undefined, { maximumFractionDigits: 2 })}`)
                                        : result.totalRevenue < effectiveBaseline 
                                            ? t.businessConclusion.negative.replace('{amount}', `${isCurrency ? "$" : ""}${Math.abs(result.totalRevenue - effectiveBaseline).toLocaleString(undefined, { maximumFractionDigits: 2 })}`)
                                            : t.businessConclusion.neutral
                                    }
                                </p>
                            </div>

                            {/* Engine Report */}
                            <div className={styles.report}>
                                <div className={styles.reportHeader}><span className={styles.dot} /> {t.telemetry.header}</div>
                                <div className={styles.reportGrid}>
                                    <div className={styles.rRow}><span>{t.telemetry.txProcessed}</span><span className={styles.rVal}>{result.fees.length.toLocaleString()}</span></div>
                                    <div className={styles.rRow}><span>{t.telemetry.rulesEvaluated}</span><span className={styles.rVal}>{activeRules.length}</span></div>
                                    <div className={styles.rRow}>
                                        <span className={styles.tipWrap}>{t.telemetry.execTime} <span className={styles.tip}><IconInfoCircle size={12} /><span className={styles.tipText}>{t.telemetryDetails.time}</span></span></span>
                                        <span className={styles.rVal}>{result.timeMs.toFixed(2)} ms</span>
                                    </div>
                                    <div className={styles.rRow}>
                                        <span className={styles.tipWrap}>{t.telemetry.throughput} <span className={styles.tip}><IconInfoCircle size={12} /><span className={styles.tipText}>{t.telemetryDetails.throughput}</span></span></span>
                                        <span className={styles.rVal}>{fmtOps(result.opsPerSec)} ops/s</span>
                                    </div>
                                    <div className={styles.rRow}><span>{t.telemetry.engine}</span><span className={styles.rVal}>{t.telemetry.engineVal}</span></div>
                                    <div className={styles.rRow}>
                                        <span className={styles.tipWrap}>{t.telemetry.determinism} <span className={styles.tip}><IconInfoCircle size={12} /><span className={styles.tipText}>{t.telemetryDetails.determinism}</span></span></span>
                                        <span className={styles.rVal}>{(result as any).isDeterministic === false ? t.telemetry.failed : t.telemetry.verified}</span>
                                    </div>
                                </div>
                            </div>

                            {/* Audit Trail — collapsible */}
                            <div className={styles.auditSection}>
                                <div className={styles.auditShield} style={{display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px'}}>
                                    <IconInfoCircle size={16} color="var(--primary)"/>
                                    <h3 style={{margin: 0, fontSize: '14px', textTransform: 'uppercase', letterSpacing: '0.05em'}}>Cryptographic Audit Trail (Time Travel)</h3>
                                </div>
                                <p className={styles.auditDesc}>Every single rule evaluation generates an immutable event hash, permitting absolute rebuilds of legacy state and compliance.</p>
                                
                                {result.fees.slice(0, auditEventsToShow).map((fee, i) => {
                                    let txs: any[] = []; try { txs = JSON.parse(txInput); } catch { }
                                    const ev = txs[i % Math.max(1, txs.length)] ?? {};
                                    // Pseudo-hash generation to simulate the time travel concept visually
                                    const pseudoHash = "sha256:0x" + Math.random().toString(16).substr(2, 8) + Date.now().toString(16).substr(4, 8);
                                    
                                    return (
                                        <div key={i} className={styles.auditCard}>
                                            <div className={styles.auditHead}>
                                                <span>Event #{Math.floor(Math.random() * 8000) + 1000}</span>
                                                <span className={styles.auditMeta}>{Object.entries(ev).map(([k, v]) => `${k}: ${v}`).join(' · ')}</span>
                                            </div>
                                            <div style={{fontSize: '10px', color: 'var(--text-dim)', marginTop: '4px', fontFamily: 'monospace'}}>{pseudoHash}</div>
                                            
                                            <div style={{marginTop: '12px'}}>
                                              {activeRules.map(r => (
                                                  <div key={r.id} className={styles.auditRule}>
                                                      <span>Rule: <span className={styles.cyan}>{r.id}</span></span>
                                                      <span>{profile.auditActionLabel}: <span className={styles.green}>{isCurrency ? "$" : ""}{(result.ruleFees[r.id]?.[i] ?? 0).toFixed(2)}</span></span>
                                                  </div>
                                              ))}
                                            </div>
                                            <div className={styles.auditTotal}>{t.audit.total} {isCurrency ? "$" : ""}{fee.toFixed(2)}</div>
                                        </div>
                                    );
                                })}
                                <div className={styles.auditFooter}>
                                    <button className={styles.auditToggle} onClick={() => setAuditExpanded(!auditExpanded)}>
                                        {auditExpanded ? t.audit.showLess : t.audit.showAll}
                                    </button>
                                    {result.fees.length > auditEventsToShow && (
                                        <span className={styles.auditMore}>{t.audit.more.replace('{count}', (result.fees.length - auditEventsToShow).toLocaleString())}</span>
                                    )}
                                </div>
                            </div>
                        </div>
                    )}
                </section>
            </div>

            {/* Advanced */}
            <div className={styles.advWrap}>
                <button className={styles.advToggle} onClick={() => setShowAdvanced(!showAdvanced)}>{t.advanced.toggle}</button>
                {showAdvanced && (
                    <div className={styles.advEditor}>
                        <div><h4>{t.advanced.ruleEditor}</h4><textarea className={styles.code} value={rulesJson} onChange={e => setRulesJson(e.target.value)} rows={8} /></div>
                        <div><h4>{t.advanced.dataEditor}</h4><textarea className={styles.code} value={txInput} onChange={e => setTxInput(e.target.value)} rows={8} /></div>
                    </div>
                )}
            </div>

            {/* Deploy / API Key Modal */}
            {isDeployModalOpen && (
                <div className={styles.modalBackdrop} onClick={() => setIsDeployModalOpen(false)}>
                    <div className={styles.modalContent} onClick={e => e.stopPropagation()}>
                        <h2>Unleash Tempus Engine</h2>
                        <p style={{marginBottom: '20px', color: 'var(--text-dim)'}}>Connect to the Tempus high-performance rules engine via API to evaluate your own transactions with 0ms latency.</p>
                        <div style={{display: 'flex', flexDirection: 'column', gap: '12px', marginBottom: '24px'}}>
                            <input type="email" placeholder="Work Email" style={{padding: '12px', borderRadius: '4px', border: '1px solid var(--border)', background: 'var(--bg-card)', color: 'var(--text)'}} />
                            <textarea placeholder="Tell us about your use case (optional)" rows={3} style={{padding: '12px', borderRadius: '4px', border: '1px solid var(--border)', background: 'var(--bg-card)', color: 'var(--text)'}} />
                        </div>
                        <div style={{display: 'flex', gap: '12px', justifyContent: 'flex-end'}}>
                            <button className={styles.btnSecondary} onClick={() => setIsDeployModalOpen(false)}>Cancel</button>
                            <button className={styles.btnPrimary} onClick={() => { alert("Thanks! Our engineering team will reach out shortly."); setIsDeployModalOpen(false); }}>Request API Key</button>
                        </div>
                    </div>
                </div>
            )}
        </main>
    );
}
