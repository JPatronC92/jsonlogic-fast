export type Language = 'en' | 'es';

export const dict = {
    en: {
        brand: "Tempus",
        tagline: "Simulate before you ship",
        tiers: { financial: "Financial", technical: "Technical" },
        steps: {
            scenario: "Scenario",
            params: "Parameters",
            run: "Run"
        },
        volume: "Volume",
        runBtn: "⚡ Run",
        runBtnRunning: "Running…",
        engineNote: "Rust + WebAssembly · 0ms latency",
        empty: {
            headline: "Simulate the financial impact of your business rules",
            sub: "Pick a scenario, adjust the parameters, and hit Run.",
            stat1label: "Revenue Impact",
            stat1val: "$—",
            stat2label: "Throughput",
            stat2val: "— ops/s",
            stat3label: "Determinism",
            stat3val: "—"
        },
        results: {
            title: "Impact Analysis",
            baseline: "Baseline",
            projected: "Projected",
            delta: "Delta"
        },
        telemetry: {
            header: "Engine Report",
            txProcessed: "Transactions",
            rulesEvaluated: "Rules",
            execTime: "Time",
            throughput: "Throughput",
            throughputTip: "Black-Friday level volume capacity with zero latency.",
            engine: "Engine",
            engineVal: "Rust + Wasm",
            determinism: "Determinism",
            determinismTip: "Same input → same output. Always.",
            verified: "✓ Verified",
            failed: "✗ Failed"
        },
        audit: {
            title: "Audit Trail",
            desc: "Deterministic trace of every {action} applied.",
            event: "Event",
            rule: "Rule",
            condition: "Condition",
            total: "TOTAL",
            more: "+{count} tracked",
            showAll: "Show more",
            showLess: "Collapse"
        },
        export: { btn: "↓ Export JSON" },
        deploy: { btn: "Deploy →" },
        advanced: {
            toggle: "Advanced",
            ruleEditor: "Rule (JSON)",
            dataEditor: "Data (JSON)"
        }
    },
    es: {
        brand: "Tempus",
        tagline: "Simula antes de implementar",
        tiers: { financial: "Financiero", technical: "Técnico" },
        steps: {
            scenario: "Escenario",
            params: "Parámetros",
            run: "Ejecutar"
        },
        volume: "Volumen",
        runBtn: "⚡ Ejecutar",
        runBtnRunning: "Ejecutando…",
        engineNote: "Rust + WebAssembly · 0ms latencia",
        empty: {
            headline: "Simula el impacto financiero de tus reglas de negocio",
            sub: "Elige un escenario, ajusta los parámetros y presiona Ejecutar.",
            stat1label: "Impacto",
            stat1val: "$—",
            stat2label: "Throughput",
            stat2val: "— ops/s",
            stat3label: "Determinismo",
            stat3val: "—"
        },
        results: {
            title: "Análisis de Impacto",
            baseline: "Línea Base",
            projected: "Proyección",
            delta: "Delta"
        },
        telemetry: {
            header: "Reporte del Motor",
            txProcessed: "Transacciones",
            rulesEvaluated: "Reglas",
            execTime: "Tiempo",
            throughput: "Throughput",
            throughputTip: "Volúmenes nivel Black-Friday sin latencia.",
            engine: "Motor",
            engineVal: "Rust + Wasm",
            determinism: "Determinismo",
            determinismTip: "Mismo input → mismo output. Siempre.",
            verified: "✓ Verificado",
            failed: "✗ Fallido"
        },
        audit: {
            title: "Audit Trail",
            desc: "Traza determinista de cada {action} aplicado.",
            event: "Evento",
            rule: "Regla",
            condition: "Condición",
            total: "TOTAL",
            more: "+{count} auditados",
            showAll: "Ver más",
            showLess: "Colapsar"
        },
        export: { btn: "↓ Exportar JSON" },
        deploy: { btn: "Deploy →" },
        advanced: {
            toggle: "Avanzado",
            ruleEditor: "Regla (JSON)",
            dataEditor: "Datos (JSON)"
        }
    }
};
