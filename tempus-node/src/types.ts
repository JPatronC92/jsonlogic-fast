// Defines the core types for the Tempus Node SDK

export interface PricingContext {
  amount: number;
  currency: string;
  [key: string]: any; // Permite propiedades dinámicas como country, merchant_id, etc.
}

export interface FeeBreakdown {
  rule_id: string;
  name: string;
  amount: number;
}

export interface CalculateFeeResponse {
  base_amount: number;
  calculated_fees: FeeBreakdown[];
  total_fees: number;
  net_settlement: number;
  currency: string;
  cryptographic_hash: string;
}

export interface BatchSimulateResponse {
  total_processed_volume: number;
  total_fees_collected: number;
  total_net_settlement: number;
  transactions_count: number;
  failed_transactions: number;
}

export interface CalculateRequest {
  scheme_urn: string;
  execution_date: string; // ISO format date, e.g., "2026-02-28T00:00:00Z"
  transaction: PricingContext;
}

export interface BatchSimulateRequest {
  scheme_urn: string;
  execution_date: string; // ISO format date
  transactions: PricingContext[];
}
