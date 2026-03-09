from pydantic import BaseModel, Field
from typing import Dict, Any, List
from datetime import datetime


class FeeBreakdown(BaseModel):
    rule_id: str = Field(description="UUID de la regla aplicada")
    name: str = Field(description="Nombre del cargo (Ej: Comisión Base 1.5%)")
    amount: float = Field(description="Monto calculado de la comisión")


class CalculateFeeRequest(BaseModel):
    scheme_urn: str = Field(
        ...,
        description="El identificador del esquema de precios, ej: urn:pricing:marketplace:mx",
    )
    execution_date: datetime = Field(
        ..., description="Fecha histórica de la transacción para el Time-Travel Audit"
    )
    transaction: Dict[str, Any] = Field(
        ..., description="Payload dinámico (monto, método de pago, etc)"
    )


class CalculateFeeResponse(BaseModel):
    base_amount: float = Field(description="Monto base sobre el que se calculó")
    calculated_fees: List[FeeBreakdown] = Field(
        description="Desglose detallado de todas las comisiones aplicadas"
    )
    total_fees: float = Field(description="Suma total de comisiones")
    net_settlement: float = Field(description="Monto neto a liquidar (base - fees)")
    currency: str = Field(description="Moneda de la transacción")
    cryptographic_hash: str = Field(
        description="Sello criptográfico de la versión de reglas usada"
    )


class BatchSimulateRequest(BaseModel):
    scheme_urn: str = Field(..., description="El identificador del esquema de precios.")
    execution_date: datetime = Field(..., description="Fecha histórica a simular.")
    transactions: List[Dict[str, Any]] = Field(
        ..., description="Lista de transacciones a procesar (Lote)."
    )


class BatchSimulateResponse(BaseModel):
    total_processed_volume: float = Field(
        description="Volumen total de transacciones procesadas"
    )
    total_fees_collected: float = Field(
        description="Suma total de comisiones cobradas en el lote"
    )
    total_net_settlement: float = Field(
        description="Suma total del monto neto a liquidar"
    )
    transactions_count: int = Field(
        description="Número de transacciones exitosas procesadas"
    )
    failed_transactions: int = Field(
        description="Número de transacciones que fallaron (ej. Payload malformado)"
    )
    currency: str = Field(default="MXN", description="Moneda principal del lote")
