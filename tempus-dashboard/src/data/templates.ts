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
    rule: object;
    params: RuleParam[];
}

// Layer 1: Dataset
export interface Dataset {
    type: 'transaction' | 'prompt' | 'api_request' | 'customer_order' | 'subscription' | 'claim' | 'shipment';
    data: any[];
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
//  TIER 1 — FINANCIAL (B2B + Fintech)
// ═══════════════════════════════════════════════════════════════

const MARKETPLACE_3TIER: PricingTemplate = {
    id: "marketplace-3tier",
    name: "Marketplace 3-Tier",
    description: "Comisión escalonada: bajo → medio → alto volumen",
    icon: "🏪",
    dataset: {
        type: 'transaction',
        data: [
            { amount: 12.50 }, { amount: 24.99 }, { amount: 15.00 }, { amount: 35.50 }, { amount: 8.99 },
            { amount: 45.00 }, { amount: 19.99 }, { amount: 29.90 }, { amount: 55.00 }, { amount: 14.50 },
            { amount: 89.99 }, { amount: 120.00 }, { amount: 150.50 }, { amount: 249.99 }, { amount: 320.00 },
            { amount: 450.00 }, { amount: 680.00 }, { amount: 850.00 }, { amount: 1200.00 }, { amount: 2450.50 },
            { amount: 3500.00 }, { amount: 5400.00 }, { amount: 8900.00 }, { amount: 14500.00 }, { amount: 28000.00 }
        ]
    },
    presentation_profile: {
        en: {
            title: "Expected Financial Impact",
            metrics: { baselineLabel: "Baseline Revenue", newLabel: "New Configuration", deltaLabel: "Revenue Delta" },
            chartLabels: { base: "payout", highlight: "fee" },
            auditActionLabel: "Applied Fee",
            units: "currency",
            reportLanguage: "financial"
        },
        es: {
            title: "Impacto Financiero Esperado",
            metrics: { baselineLabel: "Ingresos Base", newLabel: "Nueva Configuración", deltaLabel: "Impacto (Delta)" },
            chartLabels: { base: "payout", highlight: "fee" },
            auditActionLabel: "Fee Aplicado",
            units: "currency",
            reportLanguage: "financial"
        }
    },
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
};

const SAAS_SUBSCRIPTION: PricingTemplate = {
    id: "saas-subscription",
    name: "SaaS Subscription Tiers",
    description: "Facturación mensual basada en usuarios y API calls",
    icon: "💳",
    dataset: {
        type: 'subscription',
        data: [
            { monthly_users: 5, api_calls: 800, plan: "starter" },
            { monthly_users: 12, api_calls: 3500, plan: "starter" },
            { monthly_users: 25, api_calls: 12000, plan: "growth" },
            { monthly_users: 50, api_calls: 45000, plan: "growth" },
            { monthly_users: 8, api_calls: 1500, plan: "starter" },
            { monthly_users: 100, api_calls: 120000, plan: "enterprise" },
            { monthly_users: 35, api_calls: 28000, plan: "growth" },
            { monthly_users: 200, api_calls: 500000, plan: "enterprise" },
            { monthly_users: 3, api_calls: 200, plan: "starter" },
            { monthly_users: 75, api_calls: 85000, plan: "growth" },
            { monthly_users: 150, api_calls: 250000, plan: "enterprise" },
            { monthly_users: 18, api_calls: 6000, plan: "starter" },
            { monthly_users: 60, api_calls: 55000, plan: "growth" },
            { monthly_users: 300, api_calls: 900000, plan: "enterprise" },
            { monthly_users: 42, api_calls: 38000, plan: "growth" },
        ]
    },
    presentation_profile: {
        en: {
            title: "Expected Monthly Revenue",
            metrics: { baselineLabel: "Current MRR", newLabel: "New Pricing MRR", deltaLabel: "MRR Delta" },
            chartLabels: { base: "base plan", highlight: "overage charges" },
            auditActionLabel: "Monthly Charge",
            units: "currency",
            reportLanguage: "financial"
        },
        es: {
            title: "Ingresos Mensuales Esperados",
            metrics: { baselineLabel: "MRR Actual", newLabel: "MRR Nueva Tarifa", deltaLabel: "Variación MRR" },
            chartLabels: { base: "plan base", highlight: "cargos por exceso" },
            auditActionLabel: "Cargo Mensual",
            units: "currency",
            reportLanguage: "financial"
        }
    },
    rules: [
        {
            id: "saas-base-plan",
            name: "Plan Base Fee",
            version: "v1.0",
            description: "Tarifa mensual según plan contratado",
            rule: {
                "if": [
                    { "==": [{ "var": "plan" }, "enterprise"] },
                    499,
                    {
                        "if": [
                            { "==": [{ "var": "plan" }, "growth"] },
                            99,
                            29
                        ]
                    }
                ]
            },
            params: [
                { key: "starter_price", label: "Starter $/mo", value: 29, min: 9, max: 59, step: 5, suffix: "$" },
                { key: "growth_price", label: "Growth $/mo", value: 99, min: 49, max: 199, step: 10, suffix: "$" },
                { key: "enterprise_price", label: "Enterprise $/mo", value: 499, min: 199, max: 999, step: 50, suffix: "$" },
            ],
        },
        {
            id: "saas-api-overage",
            name: "API Overage Charge",
            version: "v1.0",
            description: "Cobro por exceso de API calls según plan",
            rule: {
                "if": [
                    {
                        "and": [
                            { "==": [{ "var": "plan" }, "starter"] },
                            { ">": [{ "var": "api_calls" }, 1000] }
                        ]
                    },
                    { "*": [{ "-": [{ "var": "api_calls" }, 1000] }, 0.005] },
                    {
                        "if": [
                            {
                                "and": [
                                    { "==": [{ "var": "plan" }, "growth"] },
                                    { ">": [{ "var": "api_calls" }, 50000] }
                                ]
                            },
                            { "*": [{ "-": [{ "var": "api_calls" }, 50000] }, 0.002] },
                            0
                        ]
                    }
                ]
            },
            params: [
                { key: "starter_limit", label: "Starter API Limit", value: 1000, min: 500, max: 5000, step: 500, suffix: " calls" },
                { key: "overage_rate", label: "Overage Rate", value: 0.005, min: 0.001, max: 0.02, step: 0.001, suffix: "$/call" },
            ],
        },
    ],
};

const DELIVERY_FEE: PricingTemplate = {
    id: "delivery-fee-engine",
    name: "Delivery Fee Calculator",
    description: "Costo de envío dinámico por distancia y membresía",
    icon: "🚚",
    dataset: {
        type: 'customer_order',
        data: [
            { order_total: 15.50, distance_km: 2.5, is_prime: false },
            { order_total: 42.00, distance_km: 5.0, is_prime: true },
            { order_total: 8.99, distance_km: 1.2, is_prime: false },
            { order_total: 120.00, distance_km: 12.0, is_prime: true },
            { order_total: 25.00, distance_km: 3.8, is_prime: false },
            { order_total: 55.00, distance_km: 8.5, is_prime: true },
            { order_total: 18.75, distance_km: 15.0, is_prime: false },
            { order_total: 35.00, distance_km: 4.2, is_prime: true },
            { order_total: 9.99, distance_km: 6.0, is_prime: false },
            { order_total: 200.00, distance_km: 20.0, is_prime: false },
            { order_total: 29.99, distance_km: 7.5, is_prime: false },
            { order_total: 65.00, distance_km: 3.0, is_prime: true },
            { order_total: 12.00, distance_km: 10.0, is_prime: false },
            { order_total: 88.50, distance_km: 1.5, is_prime: true },
            { order_total: 45.00, distance_km: 9.0, is_prime: false },
        ]
    },
    presentation_profile: {
        en: {
            title: "Delivery Revenue Projection",
            metrics: { baselineLabel: "Baseline Delivery Revenue", newLabel: "New Fee Structure", deltaLabel: "Revenue Delta" },
            chartLabels: { base: "free deliveries", highlight: "delivery fees" },
            auditActionLabel: "Delivery Charge",
            units: "currency",
            reportLanguage: "financial"
        },
        es: {
            title: "Proyección de Ingresos por Envío",
            metrics: { baselineLabel: "Ingresos Base por Envío", newLabel: "Nueva Estructura de Tarifas", deltaLabel: "Variación de Ingresos" },
            chartLabels: { base: "envíos gratis", highlight: "cargos por envío" },
            auditActionLabel: "Cargo por Envío",
            units: "currency",
            reportLanguage: "financial"
        }
    },
    rules: [
        {
            id: "delivery-distance-fee",
            name: "Distance-Based Fee",
            version: "v1.0",
            description: "Cargo por km de distancia de entrega",
            rule: {
                "if": [
                    {
                        "and": [
                            { "==": [{ "var": "is_prime" }, true] },
                            { ">": [{ "var": "order_total" }, 30] }
                        ]
                    },
                    0,
                    { "+": [2.99, { "*": [{ "var": "distance_km" }, 0.50] }] }
                ]
            },
            params: [
                { key: "base_fee", label: "Base Fee", value: 2.99, min: 0, max: 9.99, step: 0.50, suffix: "$" },
                { key: "per_km", label: "Per KM", value: 0.50, min: 0.10, max: 2.00, step: 0.10, suffix: "$/km" },
                { key: "free_threshold", label: "Free if order >", value: 30, min: 15, max: 100, step: 5, suffix: "$" },
            ],
        },
        {
            id: "delivery-long-distance",
            name: "Long Distance Surcharge",
            version: "v1.0",
            description: "Cargo extra por entregas fuera de zona",
            rule: {
                "if": [
                    { ">": [{ "var": "distance_km" }, 10] },
                    { "*": [{ "-": [{ "var": "distance_km" }, 10] }, 1.50] },
                    0
                ]
            },
            params: [
                { key: "distance_limit", label: "Zone Limit", value: 10, min: 5, max: 25, step: 1, suffix: " km" },
                { key: "surcharge_rate", label: "Surcharge/km", value: 1.50, min: 0.50, max: 5.00, step: 0.25, suffix: "$/km" },
            ],
        },
    ],
};

// ═══════════════════════════════════════════════════════════════
//  TIER 2 — TECHNICAL (DevOps + AI)
// ═══════════════════════════════════════════════════════════════

const AI_GUARDRAILS: PricingTemplate = {
    id: "ai-guardrails",
    name: "AI Prompt Guardrails",
    description: "Bloquea requests basados en tokens y risk_score",
    icon: "🛡️",
    dataset: {
        type: 'prompt',
        data: [
            { tokens: 150, risk_score: 0.1, is_premium: true },
            { tokens: 4000, risk_score: 0.8, is_premium: false },
            { tokens: 50, risk_score: 0.95, is_premium: false },
            { tokens: 8000, risk_score: 0.2, is_premium: true },
            { tokens: 300, risk_score: 0.05, is_premium: false },
            { tokens: 1200, risk_score: 0.4, is_premium: false },
            { tokens: 5000, risk_score: 0.99, is_premium: false },
            { tokens: 10, risk_score: 0.0, is_premium: true },
            { tokens: 6000, risk_score: 0.75, is_premium: false },
            { tokens: 250, risk_score: 0.15, is_premium: true }
        ]
    },
    presentation_profile: {
        en: {
            title: "Expected System Impact",
            metrics: { baselineLabel: "Baseline Blocked", newLabel: "Strict Policy Blocked", deltaLabel: "Block Rate Delta" },
            chartLabels: { base: "allowed", highlight: "blocked" },
            auditActionLabel: "Applied Action",
            units: "count",
            reportLanguage: "security"
        },
        es: {
            title: "Impacto en Sistemas Esperado",
            metrics: { baselineLabel: "Bloqueos Base", newLabel: "Nueva Política Estricta", deltaLabel: "Variación de Bloqueos" },
            chartLabels: { base: "permitidos", highlight: "bloqueados" },
            auditActionLabel: "Acción Aplicada",
            units: "count",
            reportLanguage: "security"
        }
    },
    rules: [
        {
            id: "risk-blocker",
            name: "High Risk Threshold",
            version: "v1.0",
            description: "Bloquea prompts con alto riesgo de inyección",
            rule: {
                "if": [
                    { ">": [{ "var": "risk_score" }, 0.8] },
                    1,
                    0
                ]
            },
            params: [
                { key: "threshold", label: "Risk Threshold", value: 0.8, min: 0.0, max: 1.0, step: 0.05, suffix: " score" },
            ],
        },
        {
            id: "token-limit",
            name: "Free Tier Token Limit",
            version: "v1.1",
            description: "Bloquea abusos de ventana de contexto",
            rule: {
                "if": [
                    {
                        "and": [
                            { "==": [{ "var": "is_premium" }, false] },
                            { ">": [{ "var": "tokens" }, 2000] }
                        ]
                    },
                    1,
                    0
                ]
            },
            params: [
                { key: "limit", label: "Max Tokens (Free)", value: 2000, min: 500, max: 8000, step: 500, suffix: " tkns" },
            ],
        }
    ]
};

const API_RATE_LIMIT: PricingTemplate = {
    id: "api-rate-limit",
    name: "Dynamic API Rate Limiting",
    description: "Throttle requests basado en CPU usage y Tier",
    icon: "⚡",
    dataset: {
        type: "api_request",
        data: [
            { current_cpu: 90, tier: "basic" },
            { current_cpu: 40, tier: "pro" },
            { current_cpu: 95, tier: "pro" },
            { current_cpu: 85, tier: "basic" },
            { current_cpu: 20, tier: "basic" },
            { current_cpu: 99, tier: "enterprise" },
            { current_cpu: 88, tier: "basic" },
            { current_cpu: 70, tier: "pro" },
            { current_cpu: 92, tier: "basic" },
            { current_cpu: 10, tier: "basic" }
        ]
    },
    presentation_profile: {
        en: {
            title: "Expected Infrastructure Impact",
            metrics: { baselineLabel: "Current Throttled", newLabel: "New Throttled", deltaLabel: "Load Shedding Delta" },
            chartLabels: { base: "served", highlight: "throttled" },
            auditActionLabel: "Routing Policy",
            units: "count",
            reportLanguage: "technical"
        },
        es: {
            title: "Impacto en Infraestructura Esperado",
            metrics: { baselineLabel: "Throttling Base", newLabel: "Throttling Aplicado", deltaLabel: "Variación de Load Shedding" },
            chartLabels: { base: "servidos", highlight: "throttled" },
            auditActionLabel: "Política de Ruteo",
            units: "count",
            reportLanguage: "technical"
        }
    },
    rules: [
        {
            id: "cpu-throttle",
            name: "CPU Throttling",
            version: "v2.0",
            description: "Drop traffic for basic tier if CPU > X",
            rule: {
                "if": [
                    {
                        "and": [
                            { "==": [{ "var": "tier" }, "basic"] },
                            { ">": [{ "var": "current_cpu" }, 80] }
                        ]
                    },
                    1,
                    0
                ]
            },
            params: [
                { key: "cpu_max", label: "Max CPU Threshold", value: 80, min: 50, max: 99, step: 5, suffix: "%" },
            ],
        }
    ]
};

// ═══════════════════════════════════════════════════════════════
//  TIER 3 — EVERYDAY / ACCESSIBLE
// ═══════════════════════════════════════════════════════════════

const LOYALTY_POINTS: PricingTemplate = {
    id: "loyalty-points",
    name: "Customer Loyalty Points",
    description: "Programa de puntos por nivel de compra",
    icon: "🌟",
    dataset: {
        type: 'customer_order',
        data: [
            { purchase_amount: 25.00, tier: "base" },
            { purchase_amount: 150.00, tier: "silver" },
            { purchase_amount: 12.50, tier: "base" },
            { purchase_amount: 450.00, tier: "gold" },
            { purchase_amount: 75.00, tier: "silver" },
            { purchase_amount: 8.99, tier: "base" },
            { purchase_amount: 320.00, tier: "gold" },
            { purchase_amount: 55.00, tier: "base" },
            { purchase_amount: 200.00, tier: "silver" },
            { purchase_amount: 18.00, tier: "base" },
            { purchase_amount: 600.00, tier: "gold" },
            { purchase_amount: 95.00, tier: "silver" },
            { purchase_amount: 35.00, tier: "base" },
            { purchase_amount: 280.00, tier: "gold" },
            { purchase_amount: 42.50, tier: "silver" },
        ]
    },
    presentation_profile: {
        en: {
            title: "Loyalty Points Projection",
            metrics: { baselineLabel: "Baseline Points", newLabel: "New Program Points", deltaLabel: "Points Delta" },
            chartLabels: { base: "base earnings", highlight: "bonus multiplier" },
            auditActionLabel: "Points Earned",
            units: "count",
            reportLanguage: "marketing"
        },
        es: {
            title: "Proyección de Puntos de Lealtad",
            metrics: { baselineLabel: "Puntos Base", newLabel: "Nuevo Programa", deltaLabel: "Variación de Puntos" },
            chartLabels: { base: "puntos base", highlight: "multiplicador bonus" },
            auditActionLabel: "Puntos Ganados",
            units: "count",
            reportLanguage: "marketing"
        }
    },
    rules: [
        {
            id: "loyalty-multiplier",
            name: "Tier Multiplier",
            version: "v1.0",
            description: "Multiplicador de puntos según nivel del cliente",
            rule: {
                "if": [
                    { "==": [{ "var": "tier" }, "gold"] },
                    { "*": [{ "var": "purchase_amount" }, 3] },
                    {
                        "if": [
                            { "==": [{ "var": "tier" }, "silver"] },
                            { "*": [{ "var": "purchase_amount" }, 2] },
                            { "*": [{ "var": "purchase_amount" }, 1] }
                        ]
                    }
                ]
            },
            params: [
                { key: "gold_mult", label: "Gold Multiplier", value: 3, min: 1, max: 5, step: 0.5, suffix: "x" },
                { key: "silver_mult", label: "Silver Multiplier", value: 2, min: 1, max: 4, step: 0.5, suffix: "x" },
                { key: "base_mult", label: "Base Multiplier", value: 1, min: 0.5, max: 2, step: 0.5, suffix: "x" },
            ],
        },
        {
            id: "loyalty-bonus",
            name: "High-Spend Bonus",
            version: "v1.0",
            description: "Bonus extra para compras grandes",
            rule: {
                "if": [
                    { ">": [{ "var": "purchase_amount" }, 200] },
                    50,
                    0
                ]
            },
            params: [
                { key: "bonus_threshold", label: "Min. Spend", value: 200, min: 50, max: 500, step: 25, suffix: "$" },
                { key: "bonus_points", label: "Bonus Points", value: 50, min: 10, max: 200, step: 10, suffix: " pts" },
            ],
        },
    ],
};

const INSURANCE_DEDUCTIBLE: PricingTemplate = {
    id: "insurance-deductible",
    name: "Health Insurance Deductible",
    description: "Calcula cobertura de reclamaciones médicas",
    icon: "🏥",
    dataset: {
        type: 'claim',
        data: [
            { claim_amount: 150.00, is_covered: true, deductible_met: true },
            { claim_amount: 2500.00, is_covered: true, deductible_met: false },
            { claim_amount: 45.00, is_covered: false, deductible_met: true },
            { claim_amount: 800.00, is_covered: true, deductible_met: true },
            { claim_amount: 12000.00, is_covered: true, deductible_met: true },
            { claim_amount: 350.00, is_covered: true, deductible_met: false },
            { claim_amount: 75.00, is_covered: false, deductible_met: false },
            { claim_amount: 5000.00, is_covered: true, deductible_met: true },
            { claim_amount: 200.00, is_covered: true, deductible_met: false },
            { claim_amount: 1800.00, is_covered: true, deductible_met: true },
            { claim_amount: 950.00, is_covered: false, deductible_met: true },
            { claim_amount: 3200.00, is_covered: true, deductible_met: true },
            { claim_amount: 125.00, is_covered: true, deductible_met: false },
            { claim_amount: 6500.00, is_covered: true, deductible_met: true },
            { claim_amount: 420.00, is_covered: true, deductible_met: true },
        ]
    },
    presentation_profile: {
        en: {
            title: "Claims Coverage Projection",
            metrics: { baselineLabel: "Baseline Covered", newLabel: "New Policy Covered", deltaLabel: "Coverage Delta" },
            chartLabels: { base: "patient pays", highlight: "insurer pays" },
            auditActionLabel: "Coverage Amount",
            units: "currency",
            reportLanguage: "financial"
        },
        es: {
            title: "Proyección de Cobertura de Reclamaciones",
            metrics: { baselineLabel: "Cobertura Base", newLabel: "Nueva Póliza", deltaLabel: "Variación de Cobertura" },
            chartLabels: { base: "paga paciente", highlight: "paga aseguradora" },
            auditActionLabel: "Monto Cubierto",
            units: "currency",
            reportLanguage: "financial"
        }
    },
    rules: [
        {
            id: "insurance-coverage",
            name: "Coverage Eligibility",
            version: "v1.0",
            description: "Determina si la aseguradora cubre la reclamación",
            rule: {
                "if": [
                    {
                        "and": [
                            { "==": [{ "var": "is_covered" }, true] },
                            { "==": [{ "var": "deductible_met" }, true] }
                        ]
                    },
                    { "*": [{ "var": "claim_amount" }, 0.80] },
                    0
                ]
            },
            params: [
                { key: "coverage_pct", label: "Coverage %", value: 80, min: 50, max: 100, step: 5, suffix: "%" },
            ],
        },
        {
            id: "insurance-copay",
            name: "Copay Calculation",
            version: "v1.0",
            description: "Copago mínimo que siempre paga el paciente",
            rule: {
                "if": [
                    {
                        "and": [
                            { "==": [{ "var": "is_covered" }, true] },
                            { "==": [{ "var": "deductible_met" }, true] }
                        ]
                    },
                    25,
                    0
                ]
            },
            params: [
                { key: "copay", label: "Copay Amount", value: 25, min: 10, max: 100, step: 5, suffix: "$" },
            ],
        },
    ],
};

const CUSTOMS_DUTY: PricingTemplate = {
    id: "customs-import-duty",
    name: "Customs Import Duty",
    description: "Aranceles de importación por categoría y tratado comercial",
    icon: "🛃",
    dataset: {
        type: 'shipment',
        data: [
            { declared_value: 1200, product_category: "electronics", origin: "China", weight_kg: 5 },
            { declared_value: 450, product_category: "textiles", origin: "Vietnam", weight_kg: 12 },
            { declared_value: 8500, product_category: "machinery", origin: "USA", weight_kg: 250 },
            { declared_value: 320, product_category: "food", origin: "Canada", weight_kg: 30 },
            { declared_value: 2800, product_category: "electronics", origin: "South Korea", weight_kg: 8 },
            { declared_value: 15000, product_category: "machinery", origin: "Germany", weight_kg: 500 },
            { declared_value: 600, product_category: "textiles", origin: "USA", weight_kg: 20 },
            { declared_value: 180, product_category: "food", origin: "Guatemala", weight_kg: 50 },
            { declared_value: 3500, product_category: "electronics", origin: "Japan", weight_kg: 3 },
            { declared_value: 950, product_category: "textiles", origin: "India", weight_kg: 15 },
            { declared_value: 22000, product_category: "machinery", origin: "USA", weight_kg: 800 },
            { declared_value: 750, product_category: "food", origin: "Chile", weight_kg: 100 },
            { declared_value: 4200, product_category: "electronics", origin: "Taiwan", weight_kg: 6 },
            { declared_value: 1100, product_category: "textiles", origin: "Canada", weight_kg: 25 },
            { declared_value: 280, product_category: "food", origin: "USA", weight_kg: 40 },
        ]
    },
    presentation_profile: {
        en: {
            title: "Import Duty Projection",
            metrics: { baselineLabel: "Current Duties Collected", newLabel: "New Tariff Structure", deltaLabel: "Duty Revenue Delta" },
            chartLabels: { base: "duty-free value", highlight: "duties collected" },
            auditActionLabel: "Import Duty",
            units: "currency",
            reportLanguage: "financial"
        },
        es: {
            title: "Proyección de Aranceles de Importación",
            metrics: { baselineLabel: "Aranceles Actuales", newLabel: "Nueva Estructura Arancelaria", deltaLabel: "Variación de Aranceles" },
            chartLabels: { base: "valor libre de arancel", highlight: "aranceles cobrados" },
            auditActionLabel: "Arancel Aplicado",
            units: "currency",
            reportLanguage: "financial"
        }
    },
    rules: [
        {
            id: "customs-ad-valorem",
            name: "Ad-Valorem Tariff",
            version: "v1.0",
            description: "Arancel % sobre valor declarado según categoría de producto",
            rule: {
                "if": [
                    { "==": [{ "var": "product_category" }, "electronics"] },
                    { "*": [{ "var": "declared_value" }, 0.15] },
                    {
                        "if": [
                            { "==": [{ "var": "product_category" }, "textiles"] },
                            { "*": [{ "var": "declared_value" }, 0.20] },
                            {
                                "if": [
                                    { "==": [{ "var": "product_category" }, "food"] },
                                    { "*": [{ "var": "declared_value" }, 0.10] },
                                    { "*": [{ "var": "declared_value" }, 0.05] }
                                ]
                            }
                        ]
                    }
                ]
            },
            params: [
                { key: "electronics_rate", label: "Electronics Rate", value: 15, min: 0, max: 35, step: 1, suffix: "%" },
                { key: "textiles_rate", label: "Textiles Rate", value: 20, min: 0, max: 35, step: 1, suffix: "%" },
                { key: "food_rate", label: "Food Rate", value: 10, min: 0, max: 25, step: 1, suffix: "%" },
                { key: "machinery_rate", label: "Machinery Rate", value: 5, min: 0, max: 20, step: 1, suffix: "%" },
            ],
        },
        {
            id: "customs-trade-agreement",
            name: "Trade Agreement Discount (T-MEC)",
            version: "v1.0",
            description: "Descuento arancelario para países con TLC (USA, Canada)",
            rule: {
                "if": [
                    {
                        "or": [
                            { "==": [{ "var": "origin" }, "USA"] },
                            { "==": [{ "var": "origin" }, "Canada"] }
                        ]
                    },
                    { "*": [{ "var": "declared_value" }, -0.05] },
                    0
                ]
            },
            params: [
                { key: "tlc_discount", label: "TLC Discount", value: 5, min: 0, max: 15, step: 1, suffix: "%" },
            ],
        },
    ],
};

// ═══════════════════════════════════════════════════════════════
//  EXPORT ALL TEMPLATES
// ═══════════════════════════════════════════════════════════════

export const TEMPLATES: PricingTemplate[] = [
    // Tier 1 — Financial
    MARKETPLACE_3TIER,
    SAAS_SUBSCRIPTION,
    DELIVERY_FEE,
    CUSTOMS_DUTY,
    // Tier 2 — Technical
    AI_GUARDRAILS,
    API_RATE_LIMIT,
];

// Rebuild rule from edited params (used by Clone & Modify)
export function rebuildRule(ruleId: string, params: RuleParam[]): object {
    const p = Object.fromEntries(params.map(p => [p.key, p.value]));

    switch (ruleId) {
        // ── Tier 1: Financial ──
        case "base-commission":
            return tieredRule({
                low: (p.low ?? 3.5) / 100,
                mid: (p.mid ?? 2.5) / 100,
                high: (p.high ?? 1.5) / 100,
                threshLow: 1000,
                threshHigh: 10000,
            });
        case "platform-fee":
            return { "+": [{ "*": [{ "var": "amount" }, 0] }, p.fixed ?? 0.30] };
        case "saas-base-plan":
            return {
                "if": [
                    { "==": [{ "var": "plan" }, "enterprise"] },
                    p.enterprise_price ?? 499,
                    {
                        "if": [
                            { "==": [{ "var": "plan" }, "growth"] },
                            p.growth_price ?? 99,
                            p.starter_price ?? 29
                        ]
                    }
                ]
            };
        case "saas-api-overage":
            return {
                "if": [
                    {
                        "and": [
                            { "==": [{ "var": "plan" }, "starter"] },
                            { ">": [{ "var": "api_calls" }, p.starter_limit ?? 1000] }
                        ]
                    },
                    { "*": [{ "-": [{ "var": "api_calls" }, p.starter_limit ?? 1000] }, p.overage_rate ?? 0.005] },
                    {
                        "if": [
                            {
                                "and": [
                                    { "==": [{ "var": "plan" }, "growth"] },
                                    { ">": [{ "var": "api_calls" }, 50000] }
                                ]
                            },
                            { "*": [{ "-": [{ "var": "api_calls" }, 50000] }, p.overage_rate ?? 0.002] },
                            0
                        ]
                    }
                ]
            };
        case "delivery-distance-fee":
            return {
                "if": [
                    {
                        "and": [
                            { "==": [{ "var": "is_prime" }, true] },
                            { ">": [{ "var": "order_total" }, p.free_threshold ?? 30] }
                        ]
                    },
                    0,
                    { "+": [p.base_fee ?? 2.99, { "*": [{ "var": "distance_km" }, p.per_km ?? 0.50] }] }
                ]
            };
        case "delivery-long-distance":
            return {
                "if": [
                    { ">": [{ "var": "distance_km" }, p.distance_limit ?? 10] },
                    { "*": [{ "-": [{ "var": "distance_km" }, p.distance_limit ?? 10] }, p.surcharge_rate ?? 1.50] },
                    0
                ]
            };

        // ── Tier 2: Technical ──
        case "risk-blocker":
            return {
                "if": [
                    { ">": [{ "var": "risk_score" }, p.threshold ?? 0.8] },
                    1,
                    0
                ]
            };
        case "token-limit":
            return {
                "if": [
                    {
                        "and": [
                            { "==": [{ "var": "is_premium" }, false] },
                            { ">": [{ "var": "tokens" }, p.limit ?? 2000] }
                        ]
                    },
                    1,
                    0
                ]
            };
        case "cpu-throttle":
            return {
                "if": [
                    {
                        "and": [
                            { "==": [{ "var": "tier" }, "basic"] },
                            { ">": [{ "var": "current_cpu" }, p.cpu_max ?? 80] }
                        ]
                    },
                    1,
                    0
                ]
            };

        // ── Tier 3: Everyday ──
        case "loyalty-multiplier":
            return {
                "if": [
                    { "==": [{ "var": "tier" }, "gold"] },
                    { "*": [{ "var": "purchase_amount" }, p.gold_mult ?? 3] },
                    {
                        "if": [
                            { "==": [{ "var": "tier" }, "silver"] },
                            { "*": [{ "var": "purchase_amount" }, p.silver_mult ?? 2] },
                            { "*": [{ "var": "purchase_amount" }, p.base_mult ?? 1] }
                        ]
                    }
                ]
            };
        case "loyalty-bonus":
            return {
                "if": [
                    { ">": [{ "var": "purchase_amount" }, p.bonus_threshold ?? 200] },
                    p.bonus_points ?? 50,
                    0
                ]
            };
        case "insurance-coverage":
            return {
                "if": [
                    {
                        "and": [
                            { "==": [{ "var": "is_covered" }, true] },
                            { "==": [{ "var": "deductible_met" }, true] }
                        ]
                    },
                    { "*": [{ "var": "claim_amount" }, (p.coverage_pct ?? 80) / 100] },
                    0
                ]
            };
        case "insurance-copay":
            return {
                "if": [
                    {
                        "and": [
                            { "==": [{ "var": "is_covered" }, true] },
                            { "==": [{ "var": "deductible_met" }, true] }
                        ]
                    },
                    p.copay ?? 25,
                    0
                ]
            };

        // ── Customs / Trade ──
        case "customs-ad-valorem":
            return {
                "if": [
                    { "==": [{ "var": "product_category" }, "electronics"] },
                    { "*": [{ "var": "declared_value" }, (p.electronics_rate ?? 15) / 100] },
                    {
                        "if": [
                            { "==": [{ "var": "product_category" }, "textiles"] },
                            { "*": [{ "var": "declared_value" }, (p.textiles_rate ?? 20) / 100] },
                            {
                                "if": [
                                    { "==": [{ "var": "product_category" }, "food"] },
                                    { "*": [{ "var": "declared_value" }, (p.food_rate ?? 10) / 100] },
                                    { "*": [{ "var": "declared_value" }, (p.machinery_rate ?? 5) / 100] }
                                ]
                            }
                        ]
                    }
                ]
            };
        case "customs-trade-agreement":
            return {
                "if": [
                    {
                        "or": [
                            { "==": [{ "var": "origin" }, "USA"] },
                            { "==": [{ "var": "origin" }, "Canada"] }
                        ]
                    },
                    { "*": [{ "var": "declared_value" }, -(p.tlc_discount ?? 5) / 100] },
                    0
                ]
            };

        default:
            return {};
    }
}
