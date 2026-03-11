from typing import Any, Dict, List

import httpx

from .types import (BatchSimulateRequest, BatchSimulateResponse,
                    CalculateFeeResponse, CalculateRequest)


class TempusError(Exception):
    pass


class TempusClient:
    def __init__(
        self,
        api_key: str,
        base_url: str = "http://localhost:8000/api/v1",
        timeout: float = 10.0,
    ):
        self.api_key = api_key
        self.base_url = base_url.rstrip("/")

        self._client = httpx.Client(
            base_url=self.base_url,
            timeout=timeout,
            headers={
                "Content-Type": "application/json",
                "X-API-Key": self.api_key,
                "X-Tempus-SDK": "python/0.1.0",
            },
        )

    def calculate(
        self, scheme_urn: str, execution_date: str, transaction: Dict[str, Any]
    ) -> CalculateFeeResponse:
        """
        Ejecuta el motor matemático para calcular tarifas de una sola transacción.
        """
        request_data = CalculateRequest(
            scheme_urn=scheme_urn,
            execution_date=execution_date,
            transaction=transaction,
        )

        response = self._client.post(
            "/billing/calculate", json=request_data.model_dump()
        )

        if not response.is_success:
            raise TempusError(f"API Error {response.status_code}: {response.text}")

        return CalculateFeeResponse.model_validate(response.json())

    def simulate_batch(
        self, scheme_urn: str, execution_date: str, transactions: List[Dict[str, Any]]
    ) -> BatchSimulateResponse:
        """
        Simula el impacto de un esquema de precios sobre un lote masivo de transacciones.
        """
        request_data = BatchSimulateRequest(
            scheme_urn=scheme_urn,
            execution_date=execution_date,
            transactions=transactions,
        )

        response = self._client.post(
            "/billing/simulate-batch", json=request_data.model_dump()
        )

        if not response.is_success:
            raise TempusError(f"API Error {response.status_code}: {response.text}")

        return BatchSimulateResponse.model_validate(response.json())

    def close(self):
        self._client.close()

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
