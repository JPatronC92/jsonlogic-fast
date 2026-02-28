from fastapi import APIRouter, HTTPException
from sqlalchemy import select
from sqlalchemy.orm import joinedload
from src.interfaces.api.dependencies import SessionDep
from src.domain.services.compliance_engine import ComplianceEngineUniversal
from src.domain.models import ReglaVersion, AmbitoAplicacion, EsquemaContexto
from src.domain.schemas.evaluate import EvaluateRequest, EvaluateResponse

router = APIRouter()
engine = ComplianceEngineUniversal()

@router.post("/evaluate", response_model=EvaluateResponse)
async def evaluate_universal(request: EvaluateRequest, session: SessionDep):
    """
    Endpoint universal de evaluación de reglas con Time-Travel y Esquema Validado.
    """
    # 1. Identificar el Ámbito de Aplicación
    stmt_ambito = select(AmbitoAplicacion).where(
        AmbitoAplicacion.jurisdiccion_id == request.selector.jurisdiccion,
        AmbitoAplicacion.autoridad_id == request.selector.autoridad,
        AmbitoAplicacion.industria == request.selector.industria
    )
    result_ambito = await session.execute(stmt_ambito)
    ambito = result_ambito.scalars().first()
    
    if not ambito:
        raise HTTPException(status_code=404, detail="Ámbito de aplicación no encontrado")
        
    # 2. Identificar el Esquema del Dominio
    stmt_esquema = select(EsquemaContexto).where(
        EsquemaContexto.dominio_id == request.selector.dominio
    ).order_by(EsquemaContexto.version.desc())
    result_esquema = await session.execute(stmt_esquema)
    esquema = result_esquema.scalars().first()
    
    if not esquema:
        raise HTTPException(status_code=404, detail="Esquema de contexto no encontrado para el dominio")

    # 3. Fetch active rules for the exact date and ambito (Time-Travel)
    stmt_reglas = (
        select(ReglaVersion)
        .options(joinedload(ReglaVersion.regla))
        .where(
            ReglaVersion.ambito_id == ambito.id,
            ReglaVersion.esquema_contexto_id == esquema.id,
            ReglaVersion.vigencia.contains(request.fecha_operacion)
        )
    )
    result_reglas = await session.execute(stmt_reglas)
    active_versiones = result_reglas.scalars().all()
        
    # 4. Evaluate rules against request transaction context using Universal Engine
    resultado = engine.evaluar(request.contexto, active_versiones, esquema.schema_json)
    
    return resultado

@router.post("/check", deprecated=True)
async def check_compliance(session: SessionDep):
    """
    DEPRECATED: Use /evaluate instead.
    """
    raise HTTPException(status_code=410, detail="Endpoint migrado a /api/v1/compliance/evaluate con el nuevo Motor Universal.")
