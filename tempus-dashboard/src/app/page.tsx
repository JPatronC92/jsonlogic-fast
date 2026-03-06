"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import CountUp from "react-countup";
import { IconInfoCircle } from "@tabler/icons-react";
import { initWasm, getWasm } from "../lib/wasm";
import { TEMPLATES, PricingTemplate, PricingRule, RuleParam, rebuildRule } from "../data/templates";
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

const TIERS = [
    { key: 'financial' as const, ids: ['marketplace-3tier', 'saas-subscription', 'delivery-fee-engine', 'customs-import-duty'] },
    { key: 'technical' as const, ids: ['ai-guardrails', 'api-rate-limit'] },
];

export default function PublicSimulator() {
    const [wasmReady, setWasmReady] = useState(false);
    const [selectedTemplate, setSelectedTemplate] = useState<PricingTemplate>(TEMPLATES[0]);
    const [activeRules, setActiveRules] = useState<PricingRule[]>(TEMPLATES[0].rules);
    const [rulesJson, setRulesJson] = useState(JSON.stringify(TEMPLATES[0].rules, null, 2));
    const [txInput, setTxInput] = useState(JSON.stringify(TEMPLATES[0].dataset.data, null, 2));
    const [result, setResult] = useState<SimResult | null>(null);
    const [baselineResult, setBaselineResult] = useState<SimResult | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [txCount, setTxCount] = useState(10000);
    const [showAdvanced, setShowAdvanced] = useState(false);
    const [isRunning, setIsRunning] = useState(false);
    const [lang, setLang] = useState<Language>('en');
    const [auditExpanded, setAuditExpanded] = useState(false);

    const resultsRef = useRef<HTMLElement>(null);
    const t = dict[lang];
    const activeProfile = selectedTemplate.presentation_profile[lang];
    const isCurrency = activeProfile.units === 'currency';

    useEffect(() => { initWasm().then(() => setWasmReady(true)).catch(console.error); }, []);

    useEffect(() => {
        if (typeof window === "undefined") return;
        const params = new URLSearchParams(window.location.search);
        const tplParam = params.get("tpl");
        if (tplParam) {
            const tpl = TEMPLATES.find(t => t.id === tplParam);
            if (tpl) { setSelectedTemplate(tpl); setActiveRules(tpl.rules); setRulesJson(JSON.stringify(tpl.rules, null, 2)); setTxInput(JSON.stringify(tpl.dataset.data, null, 2)); }
        }
    }, []);

    const selectTemplate = (template: PricingTemplate) => {
        setSelectedTemplate(template);
        setActiveRules(template.rules);
        setRulesJson(JSON.stringify(template.rules, null, 2));
        setTxInput(JSON.stringify(template.dataset.data, null, 2));
        setResult(null);
        setBaselineResult(null);
        setError(null);
        setAuditExpanded(false);
    };

    const updateRuleParam = (ruleId: string, paramKey: string, newValue: number) => {
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
        setIsRunning(true);
        setAuditExpanded(false);

        setTimeout(() => {
            try {
                const wasm = getWasm();
                let transactions: { amount: number }[];
                try { transactions = JSON.parse(txInput); } catch { setError("Invalid data"); setIsRunning(false); return; }

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

                const isCurr = activeProfile.units === 'currency';
                const totalProcessed = isCurr ? expandedTx.reduce((a, b) => a + (b.amount || 0), 0) : expandedTx.length;
                const avgRate = totalProcessed > 0 ? (totalRevenue / totalProcessed) * 100 : 0;
                const opsPerSec = elapsed > 0 ? Math.round((allFees.length * rulesToEval.length) / (elapsed / 2 / 1000)) : 0;

                const newResult = {
                    fees: allFees, ruleFees, totalRevenue, ruleRevenue, totalProcessed,
                    avgFee: totalRevenue / allFees.length, avgRate,
                    timeMs: elapsed / 2, opsPerSec, timePerTx: (elapsed / 2) / allFees.length,
                    isDeterministic
                };

                setResult(prev => {
                    if (!prev && !baselineResult) setBaselineResult(newResult as unknown as SimResult);
                    return newResult as unknown as SimResult;
                });
                setIsRunning(false);
                setTimeout(() => { resultsRef.current?.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 150);
            } catch (e: any) { setError(e.message || String(e)); setIsRunning(false); }
        }, 100);
    }, [wasmReady, activeRules, rulesJson, txInput, txCount, baselineResult, activeProfile]);

    const downloadScenario = () => {
        const blob = new Blob([JSON.stringify({ template: selectedTemplate.id, rules: activeRules, transactions: JSON.parse(txInput) }, null, 2)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a'); a.href = url; a.download = `tempus-${selectedTemplate.id}.json`;
        document.body.appendChild(a); a.click(); document.body.removeChild(a); URL.revokeObjectURL(url);
    };

    const fmtOps = (n: number) => n >= 1e6 ? `${(n / 1e6).toFixed(1)}M` : n >= 1e3 ? `${(n / 1e3).toFixed(0)}K` : `${n}`;
    const txLabels: Record<number, string> = { 100: "100", 1000: "1K", 10000: "10K", 50000: "50K" };

    const auditEventsToShow = auditExpanded ? 5 : 1;

    return (
        <main className={styles.app}>
            {/* ─── NAV ─── */}
            <nav className={styles.nav}>
                <div className={styles.navLeft}>
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
                        <h3 className={styles.panelTitle}>{t.steps.scenario}</h3>
                        {TIERS.map(tier => {
                            const templates = TEMPLATES.filter(tpl => tier.ids.includes(tpl.id));
                            if (!templates.length) return null;
                            return (
                                <div key={tier.key} className={styles.tierBlock}>
                                    <span className={`${styles.tierTag} ${styles[`t_${tier.key}`]}`}>{t.tiers[tier.key]}</span>
                                    {templates.map(tpl => (
                                        <button key={tpl.id} className={`${styles.scenarioBtn} ${selectedTemplate.id === tpl.id ? styles.scenarioActive : ''}`} onClick={() => selectTemplate(tpl)}>
                                            <span className={styles.scenarioIcon}>{tpl.icon}</span>
                                            <div className={styles.scenarioText}>
                                                <span className={styles.scenarioName}>{tpl.name}</span>
                                                <span className={styles.scenarioDesc}>{tpl.description}</span>
                                            </div>
                                        </button>
                                    ))}
                                </div>
                            );
                        })}
                    </div>

                    <div className={styles.divider} />

                    <div className={styles.panelSection}>
                        <h3 className={styles.panelTitle}>{t.steps.params}</h3>
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
                                <button key={v} className={`${styles.volBtn} ${txCount === Number(v) ? styles.volActive : ''}`} onClick={() => setTxCount(Number(v))}>{label}</button>
                            ))}
                        </div>
                    </div>

                    <button className={`${styles.runBtn} ${isRunning ? styles.runBtnPulse : ''}`} onClick={runSimulation} disabled={!wasmReady || isRunning}>
                        {isRunning ? t.runBtnRunning : t.runBtn}
                    </button>
                </aside>

                {/* MAIN CANVAS */}
                <section className={styles.canvas} ref={resultsRef}>
                    {!result ? (
                        /* ─── EMPTY STATE ─── */
                        <div className={styles.emptyState}>
                            <p className={styles.emptyHeadline}>{t.empty.headline}</p>
                            <p className={styles.emptySub}>{t.empty.sub}</p>
                            <div className={styles.skeletonGrid}>
                                <div className={styles.skeletonCard}>
                                    <span className={styles.skeletonLabel}>{t.empty.stat1label}</span>
                                    <span className={styles.skeletonVal}>{t.empty.stat1val}</span>
                                </div>
                                <div className={styles.skeletonCard}>
                                    <span className={styles.skeletonLabel}>{t.empty.stat2label}</span>
                                    <span className={styles.skeletonVal}>{t.empty.stat2val}</span>
                                </div>
                                <div className={styles.skeletonCard}>
                                    <span className={styles.skeletonLabel}>{t.empty.stat3label}</span>
                                    <span className={styles.skeletonVal}>{t.empty.stat3val}</span>
                                </div>
                            </div>
                        </div>
                    ) : (
                        /* ─── RESULTS ─── */
                        <div className={styles.resultFlow}>
                            {/* Action bar at top */}
                            <div className={styles.resultHeader}>
                                <h2 className={styles.resultTitle}>{activeProfile.title}</h2>
                                <div className={styles.headerActions}>
                                    <button className={styles.btnPrimary}>{t.deploy.btn}</button>
                                    <button className={styles.btnSecondary} onClick={downloadScenario}>{t.export.btn}</button>
                                </div>
                            </div>

                            {/* Stats */}
                            <div className={styles.statsGrid}>
                                <div className={styles.stat}>
                                    <span className={styles.statLabel}>{t.results.baseline}</span>
                                    <span className={styles.statNum}>{isCurrency ? "$" : ""}<CountUp end={baselineResult?.totalRevenue ?? result.totalRevenue} duration={1} decimals={isCurrency ? 2 : 0} separator="," /></span>
                                </div>
                                <div className={`${styles.stat} ${styles.statHL}`}>
                                    <span className={styles.statLabel}>{t.results.projected}</span>
                                    <span className={styles.statNum}>{isCurrency ? "$" : ""}<CountUp end={result.totalRevenue} duration={1} decimals={isCurrency ? 2 : 0} separator="," /></span>
                                </div>
                                <div className={styles.stat}>
                                    <span className={styles.statLabel}>{t.results.delta}</span>
                                    <span className={`${styles.statNum} ${baselineResult && result.totalRevenue !== baselineResult.totalRevenue ? (result.totalRevenue > baselineResult.totalRevenue ? styles.green : styles.red) : styles.dim}`}>
                                        {baselineResult && result.totalRevenue !== baselineResult.totalRevenue
                                            ? <CountUp end={((result.totalRevenue - baselineResult.totalRevenue) / Math.max(1, baselineResult.totalRevenue)) * 100} duration={1} decimals={2} prefix={result.totalRevenue > baselineResult.totalRevenue ? "+" : ""} suffix="%" />
                                            : "0.00%"}
                                    </span>
                                </div>
                            </div>

                            {/* Breakdown Bar with labels */}
                            <div className={styles.barWrap}>
                                <div className={styles.barLabels}>
                                    <span>{activeProfile.chartLabels.highlight}</span>
                                    <span>{activeProfile.chartLabels.base}</span>
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

                            {/* Engine Report */}
                            <div className={styles.report}>
                                <div className={styles.reportHeader}><span className={styles.dot} /> {t.telemetry.header}</div>
                                <div className={styles.reportGrid}>
                                    <div className={styles.rRow}><span>{t.telemetry.txProcessed}</span><span className={styles.rVal}>{result.fees.length.toLocaleString()}</span></div>
                                    <div className={styles.rRow}><span>{t.telemetry.rulesEvaluated}</span><span className={styles.rVal}>{activeRules.length}</span></div>
                                    <div className={styles.rRow}><span>{t.telemetry.execTime}</span><span className={styles.rVal}>{result.timeMs.toFixed(2)} ms</span></div>
                                    <div className={styles.rRow}>
                                        <span className={styles.tipWrap}>{t.telemetry.throughput} <span className={styles.tip}><IconInfoCircle size={12} /><span className={styles.tipText}>{t.telemetry.throughputTip}</span></span></span>
                                        <span className={styles.rVal}>{fmtOps(result.opsPerSec)} ops/s</span>
                                    </div>
                                    <div className={styles.rRow}><span>{t.telemetry.engine}</span><span className={styles.rVal}>{t.telemetry.engineVal}</span></div>
                                    <div className={styles.rRow}>
                                        <span className={styles.tipWrap}>{t.telemetry.determinism} <span className={styles.tip}><IconInfoCircle size={12} /><span className={styles.tipText}>{t.telemetry.determinismTip}</span></span></span>
                                        <span className={styles.rVal}>{(result as any).isDeterministic === false ? t.telemetry.failed : t.telemetry.verified}</span>
                                    </div>
                                </div>
                            </div>

                            {/* Audit Trail — collapsible */}
                            <div className={styles.auditSection}>
                                <h3>{t.audit.title}</h3>
                                <p className={styles.auditDesc}>{t.audit.desc.replace('{action}', activeProfile.auditActionLabel.toLowerCase())}</p>
                                {result.fees.slice(0, auditEventsToShow).map((fee, i) => {
                                    let txs: any[] = []; try { txs = JSON.parse(txInput); } catch { }
                                    const ev = txs[i % Math.max(1, txs.length)] ?? {};
                                    return (
                                        <div key={i} className={styles.auditCard}>
                                            <div className={styles.auditHead}>
                                                <span>{t.audit.event} #{Math.floor(Math.random() * 8000) + 1000}</span>
                                                <span className={styles.auditMeta}>{Object.entries(ev).map(([k, v]) => `${k}: ${v}`).join(' · ')}</span>
                                            </div>
                                            {activeRules.map(r => (
                                                <div key={r.id} className={styles.auditRule}>
                                                    <span>{t.audit.rule} <span className={styles.cyan}>{r.id}</span></span>
                                                    <span>{activeProfile.auditActionLabel}: <span className={styles.green}>{isCurrency ? "$" : ""}{(result.ruleFees[r.id]?.[i] ?? 0).toFixed(2)}</span></span>
                                                </div>
                                            ))}
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

            {error && <div className={styles.err}>❌ {error}</div>}
        </main>
    );
}
