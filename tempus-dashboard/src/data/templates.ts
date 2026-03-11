// Pricing Templates for the Public Simulator
// Each template now supports MULTIPLE rules, EDITABLE parameters, and Persona-Driven Contexts

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
    rule: any;
    params: RuleParam[];
}

// Layer 1: Dataset
export interface Dataset {
    type: 'transaction' | 'prompt' | 'api_request' | 'customer_order' | 'subscription' | 'claim' | 'shipment';
    data: Record<string, unknown>[];
}

// Layer 3: Presentation Profile
export interface PresentationProfileData {
    title: string;
    metrics: {
        baselineLabel: string;
        newLabel: string;
        deltaLabel: string;
    };
    chartLabels: {
        base: string;
        highlight: string;
    };
    auditActionLabel: string;
    units: 'currency' | 'count' | 'score';
    reportLanguage: 'financial' | 'technical' | 'security' | 'marketing';
    context: {
        problemStatement: string;
        tempusSolution: string;
        baselineHint?: string;
        dataContext?: string;
    };
}

export interface PresentationProfile {
    en: PresentationProfileData;
    es: PresentationProfileData;
}

export interface PricingTemplate {
    id: string;
    name: string;
    description: string;
    icon: string;
    dataset: Dataset;
    rules: PricingRule[];
    presentation_profile: PresentationProfile;
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

// ═══════════════════════════════════════════════════════════════
//  UNIVERSAL DIGITAL BUSINESS TEMPLATE
// ═══════════════════════════════════════════════════════════════

const UNIVERSAL_DIGITAL_BUSINESS: PricingTemplate = {
    id: "universal-digital-business",
    name: "Digital Business Model",
    description: "Compose subscriptions, commissions, penalties & discounts in real-time.",
    icon: "⚙️",
    dataset: {
        type: 'transaction',
        data: [
            { amount: 12.50, is_international: false, user_tier: "free" },
            { amount: 24.99, is_international: true, user_tier: "premium" },
            { amount: 15.00, is_international: false, user_tier: "free" },
            { amount: 350.50, is_international: true, user_tier: "enterprise" },
            { amount: 8.99, is_international: false, user_tier: "premium" },
            { amount: 45.00, is_international: true, user_tier: "free" },
            { amount: 19.99, is_international: false, user_tier: "free" },
            { amount: 29.90, is_international: false, user_tier: "enterprise" },
            { amount: 55.00, is_international: true, user_tier: "premium" },
            { amount: 14.50, is_international: false, user_tier: "free" },
            { amount: 89.99, is_international: true, user_tier: "free" },
            { amount: 120.00, is_international: false, user_tier: "premium" },
            { amount: 150.50, is_international: false, user_tier: "enterprise" },
            { amount: 249.99, is_international: true, user_tier: "premium" },
            { amount: 320.00, is_international: false, user_tier: "free" },
            { amount: 450.00, is_international: true, user_tier: "enterprise" },
            { amount: 680.00, is_international: false, user_tier: "premium" },
            { amount: 850.00, is_international: true, user_tier: "free" },
            { amount: 1200.00, is_international: false, user_tier: "enterprise" },
            { amount: 2450.50, is_international: true, user_tier: "premium" },
            { amount: 3500.00, is_international: false, user_tier: "enterprise" },
            { amount: 5400.00, is_international: true, user_tier: "free" }
        ]
    },
    presentation_profile: {
        en: {
            title: "Universal Pricing Engine",
            metrics: { baselineLabel: "Base Revenue", newLabel: "Projected Revenue", deltaLabel: "Revenue Delta" },
            chartLabels: { base: "base fee", highlight: "dynamic fees & penalties" },
            auditActionLabel: "Rule Impact",
            units: "currency",
            reportLanguage: "financial",
            context: {
                problemStatement: "Scaling multiple pricing rules—like fixed fees, percentage cuts, and geographic penalties—usually requires messy hardcoded logic, leading to downtime and floating point rounding errors (Billing Drift).",
                tempusSolution: "Tempus evaluates dozens of complex conditional rules simultaneously across millions of transactions in 0ms with cryptographical determinism.",
                baselineHint: "e.g., Your current avg. processor fees (2.9% + 30¢)",
                dataContext: "Simulating diverse $10-$5k global payments"
            }
        },
        es: {
            title: "Motor Universal de Pricing",
            metrics: { baselineLabel: "Ingresos Base", newLabel: "Ingresos Proyectados", deltaLabel: "Variación Neta" },
            chartLabels: { base: "cobro base", highlight: "cargos dinámicos" },
            auditActionLabel: "Impacto Regla",
            units: "currency",
            reportLanguage: "financial",
            context: {
                problemStatement: "Integrar múltiples lógicas de cobro (como comisiones, recargos por riesgo y descuentos) suele requerir código espagueti con alto riesgo de caída y errores de redondeo (Billing Drift).",
                tempusSolution: "Tempus compone y procesa decenas de reglas condicionales simultáneamente sobre millones de transacciones en 0ms con determinismo total.",
                baselineHint: "Ej. Costo promedio actual con procesador (2.9% + 30¢)",
                dataContext: "Simulando pagos globales diversos de $10-$5k"
            }
        }
    },
    rules: [
        {
            id: "base_take_rate",
            name: "Platform Take Rate",
            version: "1.0",
            description: "Variable commission percentage applied to all transactions.",
            params: [
                { key: "rate", label: "Take Rate", value: 2.5, min: 0.1, max: 15.0, step: 0.1, suffix: "%" }
            ],
            rule: { "*": [{ "var": "amount" }, 0.025] }
        },
        {
            id: "fixed_processing_fee",
            name: "Fixed Processing Fee",
            version: "1.0",
            description: "A hard fixed fee applied per successful operation.",
            params: [
                { key: "fee", label: "Fixed Fee", value: 0.30, min: 0.0, max: 2.0, step: 0.05, suffix: " $" }
            ],
            rule: 0.30
        },
        {
            id: "international_risk_penalty",
            name: "International Risk Penalty",
            version: "1.2",
            description: "Conditional surcharge for international or high-risk transactions.",
            params: [
                { key: "penalty", label: "Risk Surcharge", value: 1.5, min: 0.0, max: 5.0, step: 0.1, suffix: "%" }
            ],
            rule: {
                "if": [
                    { "==": [{ "var": "is_international" }, true] },
                    { "*": [{ "var": "amount" }, 0.015] },
                    0
                ]
            }
        },
        {
            id: "enterprise_volume_discount",
            name: "Enterprise Volume Discount",
            version: "2.0",
            description: "Dynamic discount conditionally applied if user is Enterprise tier.",
            params: [
                { key: "discount", label: "Enterprise Discount", value: -1.0, min: -5.0, max: 0.0, step: 0.1, suffix: "%" }
            ],
            rule: {
                "if": [
                    { "==": [{ "var": "user_tier" }, "enterprise"] },
                    { "*": [{ "var": "amount" }, -0.01] },
                    0
                ]
            }
        }
    ]
};

// ═══════════════════════════════════════════════════════════════
//  EXPORT
// ═══════════════════════════════════════════════════════════════

export const TEMPLATES: PricingTemplate[] = [
    UNIVERSAL_DIGITAL_BUSINESS
];

// Rebuild rule from edited params (used by Clone & Modify)
export function rebuildRule(ruleId: string, params: RuleParam[]): any {
    const p = Object.fromEntries(params.map(p => [p.key, p.value]));

    switch (ruleId) {
        case "base_take_rate":
            return { "*": [{ "var": "amount" }, (p.rate ?? 2.5) / 100] };
            
        case "fixed_processing_fee":
            return p.fee !== undefined ? p.fee : 0.30;
            
        case "international_risk_penalty":
            return {
                "if": [
                    { "==": [{ "var": "is_international" }, true] },
                    { "*": [{ "var": "amount" }, (p.penalty ?? 1.5) / 100] },
                    0
                ]
            };
            
        case "enterprise_volume_discount":
            return {
                "if": [
                    { "==": [{ "var": "user_tier" }, "enterprise"] },
                    { "*": [{ "var": "amount" }, (p.discount ?? -1.0) / 100] },
                    0
                ]
            };

        default:
            return {};
    }
}
