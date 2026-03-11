from typing import Any, Dict, List

from pydantic import BaseModel


class FeeBreakdown(BaseModel):
    rule_id: str
    name: str
    amount: float


class CalculateFeeResponse(BaseModel):
    base_amount: float
    calculated_fees: List[FeeBreakdown]
    total_fees: float
    net_settlement: float
    currency: str
    cryptographic_hash: str


class BatchSimulateResponse(BaseModel):
    total_processed_volume: float
    total_fees_collected: float
    total_net_settlement: float
    transactions_count: int
    failed_transactions: int


class CalculateRequest(BaseModel):
    scheme_urn: str
    execution_date: str
    transaction: Dict[str, Any]


class BatchSimulateRequest(BaseModel):
    scheme_urn: str
    execution_date: str
    transactions: List[Dict[str, Any]]
