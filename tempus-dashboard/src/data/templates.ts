// Pricing Templates for the Public Simulator
// Each template now supports MULTIPLE rules and EDITABLE parameters

export interface RuleParam {
    key: string;
    label: string;
    value: number;
    min: number;
    max: number;
    step: number;
    suffix: string; // "%", "$", etc.
}

export interface PricingRule {
    id: string;
    name: string;
    version: string;
    description: string;
    rule: object;
    params: RuleParam[];
}

export interface PricingTemplate {
    id: string;
    name: string;
    description: string;
    icon: string;
    rules: PricingRule[];
    sampleTransactions: { amount: number }[];
}

// Helper: builds a json-logic rule from params dynamically
function tieredRule(p: { low: number; mid: number; high: number; threshLow: number; threshHigh: number }) {
    return {
        "if": [
            { ">": [{ "var": "amount" }, p.threshHigh] },
            { "*": [{ "var": "amount" }, p.high] },
            {
                "if": [
                    { ">": [{ "var": "amount" }, p.threshLow] },
                    { "*": [{ "var": "amount" }, p.mid] },
                    { "*": [{ "var": "amount" }, p.low] }
                ]
            }
        ]
    };
}

export const TEMPLATES: PricingTemplate[] = [
    {
        id: "marketplace-3tier",
        name: "Marketplace 3-Tier",
        description: "Comisión escalonada: bajo → medio → alto volumen",
        icon: "🏪",
        rules: [
            {
                id: "base-commission",
                name: "Comisión Base",
                version: "v1.0",
                description: "Comisión principal por nivel de transacción",
                rule: tieredRule({ low: 0.035, mid: 0.025, high: 0.015, threshLow: 1000, threshHigh: 10000 }),
                params: [
                    { key: "low", label: "Bajo volumen (<$1K)", value: 3.5, min: 0.5, max: 10, step: 0.5, suffix: "%" },
                    { key: "mid", label: "Medio ($1K-$10K)", value: 2.5, min: 0.5, max: 8, step: 0.5, suffix: "%" },
                    { key: "high", label: "Alto (>$10K)", value: 1.5, min: 0.1, max: 5, step: 0.1, suffix: "%" },
                ],
            },
            {
                id: "platform-fee",
                name: "Cuota de Plataforma",
                version: "v1.0",
                description: "Fee fijo por transacción procesada",
                rule: { "+": [{ "*": [{ "var": "amount" }, 0] }, 0.30] },
                params: [
                    { key: "fixed", label: "Fee fijo", value: 0.30, min: 0.05, max: 2.0, step: 0.05, suffix: "$" },
                ],
            },
        ],
        sampleTransactions: [
            // Distribución hiper-realista: 60% micro/pequeñas, 30% medianas, 10% grandes/B2B
            { amount: 12.50 }, { amount: 24.99 }, { amount: 15.00 }, { amount: 35.50 }, { amount: 8.99 },
            { amount: 45.00 }, { amount: 19.99 }, { amount: 29.90 }, { amount: 55.00 }, { amount: 14.50 },
            { amount: 89.99 }, { amount: 120.00 }, { amount: 150.50 }, { amount: 249.99 }, { amount: 320.00 },
            { amount: 450.00 }, { amount: 680.00 }, { amount: 850.00 }, { amount: 1200.00 }, { amount: 2450.50 },
            { amount: 3500.00 }, { amount: 5400.00 }, { amount: 8900.00 }, { amount: 14500.00 }, { amount: 28000.00 }
        ],
    },
    {
        id: "flat-percentage",
        name: "Flat Percentage",
        description: "Comisión fija estilo procesador de pagos",
        icon: "💳",
        rules: [
            {
                id: "processing-fee",
                name: "Processing Fee",
                version: "v2.1",
                description: "Porcentaje fijo sobre cada transacción",
                rule: { "*": [{ "var": "amount" }, 0.029] },
                params: [
                    { key: "rate", label: "Tasa", value: 2.9, min: 0.5, max: 5, step: 0.1, suffix: "%" },
                ],
            },
            {
                id: "fixed-fee",
                name: "Fixed Fee",
                version: "v1.0",
                description: "Cargo fijo adicional por transacción",
                rule: { "+": [{ "*": [{ "var": "amount" }, 0] }, 0.30] },
                params: [
                    { key: "fixed", label: "Cargo fijo", value: 0.30, min: 0.0, max: 1.0, step: 0.05, suffix: "$" },
                ],
            },
        ],
        sampleTransactions: [
            { amount: 100 }, { amount: 500 }, { amount: 1000 },
            { amount: 2500 }, { amount: 5000 }, { amount: 10000 },
            { amount: 25000 }, { amount: 50000 }, { amount: 100000 },
            { amount: 250000 },
        ],
    },
    {
        id: "saas-usage",
        name: "SaaS Usage-Based",
        description: "Precio por unidad con mínimo garantizado",
        icon: "☁️",
        rules: [
            {
                id: "per-unit",
                name: "Per-Unit Pricing",
                version: "v1.2",
                description: "Cargo por cada API call con piso mínimo",
                rule: { "max": [{ "*": [{ "var": "amount" }, 0.005] }, 1] },
                params: [
                    { key: "rate", label: "Por unidad", value: 0.005, min: 0.001, max: 0.05, step: 0.001, suffix: "$" },
                    { key: "floor", label: "Mínimo", value: 1.0, min: 0, max: 10, step: 0.5, suffix: "$" },
                ],
            },
        ],
        sampleTransactions: [
            { amount: 50 }, { amount: 100 }, { amount: 500 },
            { amount: 1000 }, { amount: 5000 }, { amount: 10000 },
            { amount: 50000 }, { amount: 100000 }, { amount: 500000 },
            { amount: 1000000 },
        ],
    },
    {
        id: "staircase-volume",
        name: "Staircase Volume",
        description: "Descuento por volumen en 4 niveles",
        icon: "📊",
        rules: [
            {
                id: "volume-discount",
                name: "Volume Discount",
                version: "v3.0",
                description: "Tasa decreciente a mayor volumen",
                rule: {
                    "if": [
                        { ">": [{ "var": "amount" }, 50000] },
                        { "*": [{ "var": "amount" }, 0.01] },
                        {
                            "if": [
                                { ">": [{ "var": "amount" }, 10000] },
                                { "*": [{ "var": "amount" }, 0.02] },
                                {
                                    "if": [
                                        { ">": [{ "var": "amount" }, 5000] },
                                        { "*": [{ "var": "amount" }, 0.025] },
                                        { "*": [{ "var": "amount" }, 0.03] }
                                    ]
                                }
                            ]
                        }
                    ]
                },
                params: [
                    { key: "tier1", label: "Default", value: 3.0, min: 1, max: 10, step: 0.5, suffix: "%" },
                    { key: "tier2", label: ">$5K", value: 2.5, min: 0.5, max: 8, step: 0.5, suffix: "%" },
                    { key: "tier3", label: ">$10K", value: 2.0, min: 0.5, max: 5, step: 0.5, suffix: "%" },
                    { key: "tier4", label: ">$50K", value: 1.0, min: 0.1, max: 3, step: 0.1, suffix: "%" },
                ],
            },
            {
                id: "min-fee",
                name: "Cuota Mínima",
                version: "v1.0",
                description: "Piso mínimo de cobro",
                rule: { "max": [{ "*": [{ "var": "amount" }, 0] }, 5] },
                params: [
                    { key: "floor", label: "Mínimo", value: 5.0, min: 0, max: 25, step: 1, suffix: "$" },
                ],
            },
        ],
        sampleTransactions: [
            { amount: 1000 }, { amount: 3000 }, { amount: 5500 },
            { amount: 8000 }, { amount: 12000 }, { amount: 25000 },
            { amount: 40000 }, { amount: 55000 }, { amount: 75000 },
            { amount: 100000 },
        ],
    },
];

// Rebuild rule from edited params (used by Clone & Modify)
export function rebuildRule(ruleId: string, params: RuleParam[]): object {
    const p = Object.fromEntries(params.map(p => [p.key, p.value]));

    switch (ruleId) {
        case "base-commission":
            return tieredRule({
                low: (p.low ?? 3.5) / 100,
                mid: (p.mid ?? 2.5) / 100,
                high: (p.high ?? 1.5) / 100,
                threshLow: 1000,
                threshHigh: 10000,
            });
        case "processing-fee":
            return { "*": [{ "var": "amount" }, (p.rate ?? 2.9) / 100] };
        case "fixed-fee":
        case "platform-fee":
            return { "+": [{ "*": [{ "var": "amount" }, 0] }, p.fixed ?? 0.30] };
        case "per-unit":
            return { "max": [{ "*": [{ "var": "amount" }, p.rate ?? 0.005] }, p.floor ?? 1] };
        case "volume-discount":
            return {
                "if": [
                    { ">": [{ "var": "amount" }, 50000] },
                    { "*": [{ "var": "amount" }, (p.tier4 ?? 1) / 100] },
                    {
                        "if": [
                            { ">": [{ "var": "amount" }, 10000] },
                            { "*": [{ "var": "amount" }, (p.tier3 ?? 2) / 100] },
                            {
                                "if": [
                                    { ">": [{ "var": "amount" }, 5000] },
                                    { "*": [{ "var": "amount" }, (p.tier2 ?? 2.5) / 100] },
                                    { "*": [{ "var": "amount" }, (p.tier1 ?? 3) / 100] }
                                ]
                            }
                        ]
                    }
                ]
            };
        case "min-fee":
            return { "max": [{ "*": [{ "var": "amount" }, 0] }, p.floor ?? 5] };
        default:
            return {};
    }
}
