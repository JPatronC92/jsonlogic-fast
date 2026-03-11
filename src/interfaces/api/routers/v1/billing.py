from fastapi import APIRouter, Depends, HTTPException

from src.core.security import get_current_tenant
from src.domain.models import Tenant
from src.domain.schemas.pricing import (
    BatchSimulateRequest,
    BatchSimulateResponse,
    CalculateFeeRequest,
    CalculateFeeResponse,
)
from src.domain.services.pricing_engine import PricingEngine
from src.infrastructure.repository import PricingRepository
from src.interfaces.api.dependencies import SessionDep

router = APIRouter()
engine = PricingEngine()


@router.post("/simulate-batch", response_model=BatchSimulateResponse)
async def simulate_batch_fees(
    request: BatchSimulateRequest,
    session: SessionDep = None,
    tenant: Tenant = Depends(get_current_tenant),
):
    """
    Simula el impacto de un esquema de precios contra un lote histórico de transacciones.
    Ideal para forecasting de CFOs y simulaciones de revenue-share.
    """
    repo = PricingRepository(session)

    scheme = await repo.get_scheme_by_urn(request.scheme_urn, tenant.id)
    if not scheme:
        raise HTTPException(
            status_code=404,
            detail=f"Esquema de precios '{request.scheme_urn}' no encontrado.",
        )

    reglas_activas = await repo.get_active_rules_for_scheme(
        request.scheme_urn, request.execution_date.date(), tenant.id
    )

    if not reglas_activas:
        raise HTTPException(
            status_code=400,
            detail=f"No hay reglas de facturación activas para '{request.scheme_urn}' en la fecha {request.execution_date.date()}.",
        )

    resultado = engine.simulate_batch(request.transactions, reglas_activas)
    return resultado


@router.post("/calculate", response_model=CalculateFeeResponse)
async def calculate_billing_fees(
    request: CalculateFeeRequest,
    session: SessionDep = None,
    tenant: Tenant = Depends(get_current_tenant),
):
    """
    Evalúa una transacción contra el esquema de precios activo en una fecha histórica específica.
    """
    repo = PricingRepository(session)

    # 1. Asegurar que el esquema existe
    scheme = await repo.get_scheme_by_urn(request.scheme_urn, tenant.id)
    if not scheme:
        raise HTTPException(
            status_code=404,
            detail=f"Esquema de precios '{request.scheme_urn}' no encontrado.",
        )

    # 2. Viaje en el tiempo: Obtener solo las reglas vigentes en esa fecha
    reglas_activas = await repo.get_active_rules_for_scheme(
        request.scheme_urn, request.execution_date.date(), tenant.id
    )

    if not reglas_activas:
        raise HTTPException(
            status_code=400,
            detail=f"No hay reglas de facturación activas para el esquema '{request.scheme_urn}' en la fecha {request.execution_date.date()}.",
        )

    # 3. Evaluación Determinista
    try:
        resultado = engine.calculate(request.transaction, reglas_activas)
        return resultado
    except ValueError as e:
        raise HTTPException(status_code=422, detail=str(e))
    except Exception:
        raise HTTPException(
            status_code=500, detail="Error interno calculando comisiones"
        )
