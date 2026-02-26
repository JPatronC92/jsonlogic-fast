from fastapi import APIRouter
from pydantic import BaseModel
from typing import Dict, Any, List
from datetime import date
from sqlalchemy import select
from sqlalchemy.orm import joinedload
from src.interfaces.api.dependencies import SessionDep
from src.domain.services.compliance_engine import ComplianceEngine
from src.domain.models import ReglaVersion, ReglaIdentidad
from src.domain.schemas.compliance import ReglaEvaluable, UniversalEvaluateRequest, ResultadoEvaluacion

router = APIRouter()
engine = ComplianceEngine()

class ComplianceCheckRequest(BaseModel):
    transaction_id: str
    metadata: Dict[str, Any]
    # More fields related to the transaction to check

class ComplianceCheckResponse(BaseModel):
    compliant: bool
    violations: List[str] = []

@router.post("/evaluate", response_model=ResultadoEvaluacion)
async def evaluate_universal(request: UniversalEvaluateRequest, session: SessionDep):
    """
    Endpoint universal de evaluación de reglas con Time-Travel.
    """
    # 1. Fetch active rules for the exact date (Time-Travel)
    stmt = (
        select(ReglaVersion)
        .options(joinedload(ReglaVersion.regla))
        .where(ReglaVersion.vigencia.contains(request.fecha_operacion))
    )
    result = await session.execute(stmt)
    active_versiones = result.scalars().all()
    
    # 2. Mapear a DTOs de evaluación
    reglas_evaluables = []
    for v in active_versiones:
        reglas_evaluables.append(
            ReglaEvaluable(
                id_version=str(v.id),
                clave_regla=v.regla.clave_interna,
                logica=v.logica_json,
                schema_validacion=v.contexto_schema,
                template_error=v.template_error,
                prioridad=v.prioridad,
                severidad=v.severidad
            )
        )
        
    # 3. Evaluate rules against request transaction context
    resultado = engine.evaluar(request.transaccion, reglas_evaluables)
    
    return resultado

@router.post("/check", response_model=ComplianceCheckResponse)
async def check_compliance(request: ComplianceCheckRequest, session: SessionDep):
    """
    Endpoint for ERPs to validate rules against a transaction.
    """
    # 1. Fetch active rules for the context
    stmt = (
        select(ReglaVersion)
        .options(joinedload(ReglaVersion.regla))
    )
    result = await session.execute(stmt)
    versiones = result.scalars().all()
    
    # Filtrar versiones activas (vigentes el día de hoy)
    today = date.today()
    active_versiones = []
    for v in versiones:
        # asyncpg Range tiene propiedades 'lower' y 'upper'
        inicio = v.vigencia.lower
        fin = v.vigencia.upper
        if inicio <= today and (fin is None or fin > today):
            active_versiones.append(v)
            
    # 2. Mapear a DTOs de evaluación
    reglas_evaluables = []
    for v in active_versiones:
        reglas_evaluables.append(
            ReglaEvaluable(
                id_version=str(v.id),
                clave_regla=v.regla.clave_interna,
                logica=v.logica_json,
                schema_validacion=v.contexto_schema,
                template_error=v.template_error,
                prioridad=v.prioridad,
                severidad=v.severidad
            )
        )
        
    # 3. Evaluate rules against request data
    resultado = engine.evaluar(request.metadata, reglas_evaluables)
    
    return ComplianceCheckResponse(
        compliant=resultado.es_valido,
        violations=resultado.errores
    )
