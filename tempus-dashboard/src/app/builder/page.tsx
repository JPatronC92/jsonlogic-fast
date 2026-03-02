"use client";

import { useState } from "react";
import styles from "./builder.module.css";

export default function RuleBuilder() {
    const [ruleName, setRuleName] = useState("");
    const [feeType, setFeeType] = useState("PERCENTAGE");
    const [conditions, setConditions] = useState<{ operator: string; variable: string; value: number; thenFee: number }[]>([]);
    const [defaultFee, setDefaultFee] = useState<number>(0);
    const [saving, setSaving] = useState(false);
    const [resultMsg, setResultMsg] = useState("");

    const addCondition = () => {
        setConditions([...conditions, { operator: ">", variable: "amount", value: 0, thenFee: 0 }]);
    };

    const removeCondition = (index: number) => {
        const newConds = [...conditions];
        newConds.splice(index, 1);
        setConditions(newConds);
    };

    const updateCondition = (index: number, field: string, val: any) => {
        const newConds = [...conditions];
        (newConds[index] as any)[field] = val;
        setConditions(newConds);
    };

    // Convert UI state to json-logic
    const generateJsonLogic = () => {
        if (conditions.length === 0) {
            if (feeType === "PERCENTAGE") {
                return { "*": [{ "var": "amount" }, defaultFee / 100] };
            }
            return defaultFee;
        }

        // Process conditions from last to first to nest the "if" statements
        let currentLogic: any = defaultFee;

        if (feeType === "PERCENTAGE") {
            currentLogic = { "*": [{ "var": "amount" }, defaultFee / 100] };
        }

        for (let i = conditions.length - 1; i >= 0; i--) {
            const cond = conditions[i];
            const conditionExp = { [cond.operator]: [{ "var": cond.variable }, cond.value] };
            const thenExp = feeType === "PERCENTAGE" ? { "*": [{ "var": "amount" }, cond.thenFee / 100] } : cond.thenFee;

            currentLogic = { "if": [conditionExp, thenExp, currentLogic] };
        }

        return currentLogic;
    };

    const saveRule = async () => {
        setSaving(true);
        setResultMsg("");
        try {
            const logic = generateJsonLogic();
            const payload = {
                rule_name: ruleName,
                fee_type: feeType,
                schema_id: "00000000-0000-0000-0000-000000000000", // Will be replaced by backend mapping
                logica_json: logic,
                vigencia_start: new Date().toISOString().split('T')[0]
            };

            const res = await fetch("/api/rules", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(payload)
            });

            if (res.ok) {
                setResultMsg("Rule saved successfully!");
            } else {
                const errorData = await res.json();
                setResultMsg(`Error: ${errorData.error || errorData.detail || "Failed to save"}`);
            }
        } catch (err: any) {
            setResultMsg(`Error: ${err.message}`);
        } finally {
            setSaving(false);
        }
    };

    return (
        <main className="container">
            <header className="header">
                <h1>Visual Rule Builder</h1>
                <p>Design complex pricing logic without writing code.</p>
            </header>

            <div className="grid-2">
                <div className="card">
                    <h2>Rule Configuration</h2>

                    <div className="input-group">
                        <label>Rule Name</label>
                        <input
                            type="text"
                            placeholder="e.g. Premium Tier Pricing"
                            value={ruleName}
                            onChange={(e: any) => setRuleName(e.target.value)}
                        />
                    </div>

                    <div className="input-group">
                        <label>Fee Type</label>
                        <select className={styles.select} value={feeType} onChange={(e: any) => setFeeType(e.target.value)}>
                            <option value="PERCENTAGE">Percentage (%)</option>
                            <option value="FIXED_FEE">Fixed Fee (Amount)</option>
                        </select>
                    </div>

                    <h3>Conditions (Tiers)</h3>
                    {conditions.map((cond, i) => (
                        <div key={i} className={styles.conditionRow}>
                            <span>If</span>
                            <select className={styles.selectSmall} value={cond.variable} onChange={(e: any) => updateCondition(i, 'variable', e.target.value)}>
                                <option value="amount">Amount</option>
                                <option value="total_volume">Total Volume</option>
                            </select>
                            <select className={styles.selectSmall} value={cond.operator} onChange={(e: any) => updateCondition(i, 'operator', e.target.value)}>
                                <option value=">">&gt;</option>
                                <option value=">=">&gt;=</option>
                                <option value="<">&lt;</option>
                                <option value="<=">&lt;=</option>
                                <option value="==">==</option>
                            </select>
                            <input type="number" className={styles.inputSmall} value={cond.value} onChange={(e: any) => updateCondition(i, 'value', Number(e.target.value))} />
                            <span>Then Fee</span>
                            <input type="number" className={styles.inputSmall} value={cond.thenFee} onChange={(e: any) => updateCondition(i, 'thenFee', Number(e.target.value))} />
                            <button className={styles.btnRemove} onClick={() => removeCondition(i)}>✕</button>
                        </div>
                    ))}

                    <button className={styles.btnAdd} onClick={addCondition}>+ Add Tier Condition</button>

                    <div className="input-group" style={{ marginTop: "2rem" }}>
                        <label>Default Fee {feeType === 'PERCENTAGE' ? '(%)' : '($)'} (If no conditions met)</label>
                        <input
                            type="number"
                            value={defaultFee}
                            onChange={(e: any) => setDefaultFee(Number(e.target.value))}
                        />
                    </div>

                    <button onClick={saveRule} disabled={saving || !ruleName} className="btn-primary" style={{ marginTop: "1rem" }}>
                        {saving ? "Saving..." : "Save Pricing Rule"}
                    </button>

                    {resultMsg && <p className={resultMsg.includes("Error") ? "error" : styles.successMsg}>{resultMsg}</p>}
                </div>

                <div className="card">
                    <h2>Generated JSON-Logic</h2>
                    <p className="desc">This is the deterministic code that will be compiled into Rust via PyO3 for ultra-fast execution.</p>
                    <pre className={styles.codePreview}>
                        {JSON.stringify(generateJsonLogic(), null, 2)}
                    </pre>
                </div>
            </div>
        </main>
    );
}
