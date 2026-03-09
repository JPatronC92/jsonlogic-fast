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
            sub: "Discover how much revenue you're leaving on the table. Adjust the parameters below to instantly simulate your financial impact.",
            stat1label: "Revenue Impact",
            stat1val: "$—",
            stat2label: "Throughput",
            stat2val: "— ops/s",
            stat3label: "Determinism",
            stat3val: "—"
        },
        results: {
            title: "Impact Analysis",
            baseline: "Base Revenue",
            projected: "Projected",
            delta: "Net Impact"
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
        },
        contextIntro: {
            problem: "The Problem",
            solution: "The Tempus Solution",
            cta: "Adjust parameters on the left and hit Run"
        },
        businessConclusion: {
            title: "Business Conclusion",
            positive: "By switching to this pricing structure, your business would generate an additional {amount} in new revenue. Tempus processes these complex rules instantly, ensuring you never drop a transaction or overcharge a customer.",
            negative: "Applying this structure results in a {amount} variance compared to your base revenue. This ensures you can deploy aggressive pricing strategies safely, knowing the exact impact before shipping.",
            neutral: "This structure maintains your base revenue while securely moving your pricing logic to a deterministic, zero-latency engine."
        },
        telemetryDetails: {
            throughput: "Supports Black-Friday traffic without sweating.",
            determinism: "Zero risk of mischarging. Same input, same output. Always.",
            time: "Doesn't slow down your website or checkout."
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
            sub: "Descubre cuántos ingresos estás dejando sobre la mesa. Ajusta los parámetros interactivos para simular al instante tu rentabilidad real.",
            stat1label: "Impacto",
            stat1val: "$—",
            stat2label: "Throughput",
            stat2val: "— ops/s",
            stat3label: "Determinismo",
            stat3val: "—"
        },
        results: {
            title: "Análisis de Impacto",
            baseline: "Ingreso Base",
            projected: "Proyección",
            delta: "Impacto Neto"
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
        },
        contextIntro: {
            problem: "El Problema",
            solution: "La Solución Tempus",
            cta: "Ajusta los parámetros a la izquierda y presiona Ejecutar"
        },
        businessConclusion: {
            title: "Conclusión de Negocio",
            positive: "Al implementar esta estructura de precios, tu negocio generaría {amount} adicionales en ingresos. Tempus procesa estas reglas al instante, asegurando que cobres exactamente lo correcto sin ralentizar tu checkout.",
            negative: "Aplicar esta estructura resulta en una variación de {amount} respecto a tu nivel base. Esto te permite desplegar estrategias agresivas con total seguridad, conociendo el impacto antes de salir a producción.",
            neutral: "Esta estructura retiene tus ingresos base alineados, pero migrando tu lógica a un motor determinista infalible."
        },
        telemetryDetails: {
            throughput: "Soporta tráfico de Black-Friday sin sudar.",
            determinism: "Cero riesgo de sobrecobrar. Mismo input, mismo cálculo. Siempre.",
            time: "No ralentiza tu página web ni tu checkout."
        }
    }
};
