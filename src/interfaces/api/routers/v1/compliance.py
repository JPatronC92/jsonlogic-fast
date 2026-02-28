from fastapi import APIRouter, HTTPException
from sqlalchemy import select
from sqlalchemy.orm import joinedload
from src.interfaces.api.dependencies import SessionDep
from src.domain.services.compliance_engine import ComplianceEngineUniversal
from src.domain.models import ReglaVersion, AmbitoAplicacion, EsquemaContexto
from src.domain.schemas.evaluate import EvaluateRequest, EvaluateResponse
from src.core.cache import rule_cache

router = APIRouter()
engine = ComplianceEngineUniversal()

@router.post("/evaluate", response_model=EvaluateResponse)
async def evaluate_universal(request: EvaluateRequest, session: SessionDep):
    """
    Endpoint universal de evaluación de reglas con Time-Travel y Esquema Validado.
    """
    # Clave de caché para evitar consultas masivas a DB
    cache_key = f"rules_{request.selector.jurisdiccion}_{request.selector.autoridad}_{request.selector.industria}_{request.selector.dominio}_{request.fecha_operacion}"
    
    cached_data = rule_cache.get(cache_key)

    if cached_data:
        active_versiones = cached_data["reglas"]
        schema_json = cached_data["esquema"]
    else:
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
        schema_json = esquema.schema_json
        
        # Guardar en caché para próximas peticiones idénticas en la ventana de TTL
        rule_cache.set(cache_key, {
            "reglas": active_versiones,
            "esquema": schema_json
        })

    # 4. Evaluate rules against request transaction context using Universal Engine
    resultado = engine.evaluar(request.contexto, active_versiones, schema_json)
    
    return resultado

@router.post("/check", deprecated=True)
async def check_compliance(session: SessionDep):
    """
    DEPRECATED: Use /evaluate instead.
    """
    raise HTTPException(status_code=410, detail="Endpoint migrado a /api/v1/compliance/evaluate con el nuevo Motor Universal.")
