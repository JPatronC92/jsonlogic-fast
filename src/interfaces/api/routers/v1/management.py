from fastapi import APIRouter, HTTPException
from sqlalchemy import select
from asyncpg import Range
from src.interfaces.api.dependencies import SessionDep
from src.domain.models import EsquemaContexto, ReglaIdentidad, ReglaVersion
from uuid import UUID
from sqlalchemy.orm import joinedload
from src.domain.schemas.management import (
    EsquemaContextoCreate, EsquemaContextoResponse,
    ReglaIdentidadCreate, ReglaIdentidadResponse,
    ReglaVersionCreate, ReglaVersionResponse,
    VerifyRuleResponse
)
from src.domain.services.crypto import canonicalize_payload, sha256_hash
from datetime import datetime, timezone

router = APIRouter(tags=["management"])

@router.post("/esquemas", response_model=EsquemaContextoResponse)
async def crear_esquema(request: EsquemaContextoCreate, session: SessionDep):
    nuevo_esquema = EsquemaContexto(
        dominio_id=request.dominio_id,
        version=request.version,
        schema_json=request.schema_json
    )
    session.add(nuevo_esquema)
    await session.commit()
    await session.refresh(nuevo_esquema)
    return nuevo_esquema

@router.post("/identidades", response_model=ReglaIdentidadResponse)
async def crear_identidad_regla(request: ReglaIdentidadCreate, session: SessionDep):
    # Validar unicidad URN
    stmt = select(ReglaIdentidad).where(ReglaIdentidad.urn_global == request.urn_global)
    result = await session.execute(stmt)
    if result.scalars().first():
        raise HTTPException(status_code=400, detail="urn_global ya existe")

    nueva_identidad = ReglaIdentidad(
        urn_global=request.urn_global,
        clave_interna=request.clave_interna,
        dominio_id=request.dominio_id,
        tipo_obligacion=request.tipo_obligacion,
        criticidad=request.criticidad
    )
    session.add(nueva_identidad)
    await session.commit()
    await session.refresh(nueva_identidad)
    return nueva_identidad

@router.post("/versiones", response_model=ReglaVersionResponse)
async def publicar_version_regla(request: ReglaVersionCreate, session: SessionDep):
    # Fetch la identidad para la firma (para tener el urn)
    identidad = await session.get(ReglaIdentidad, request.regla_uuid)
    if not identidad:
        raise HTTPException(status_code=404, detail="ReglaIdentidad no encontrada")

    # Armar vigencia
    lower = request.vigencia.inicio
    # postgres daterange typically excludes upper bound, logic to keep it simple:
    upper = request.vigencia.fin
    rango_vigencia = Range(lower, upper, bounds='[)')

    # Generar firma inmutable
    canonical_bytes = canonicalize_payload(
        urn_global=identidad.urn_global or "urn:lex:unknown",
        vigencia_desde=lower,
        vigencia_hasta=upper,
        esquema_id=str(request.esquema_contexto_id),
        logica_json=request.logica_json
    )
    firma = sha256_hash(canonical_bytes)

    nueva_version = ReglaVersion(
        regla_uuid=request.regla_uuid,
        ambito_id=request.ambito_id,
        esquema_contexto_id=request.esquema_contexto_id,
        logica_json=request.logica_json,
        template_error=request.template_error,
        vigencia=rango_vigencia,
        hash_firma=firma,
        hash_algoritmo="SHA-256",
        firmado_en=datetime.now(timezone.utc)
    )

    session.add(nueva_version)
    try:
        await session.commit()
        await session.refresh(nueva_version)
    except Exception as e:
        await session.rollback()
        # Posible error de ExcludeConstraint si las vigencias se solapan
        raise HTTPException(status_code=400, detail=f"Error al publicar versión (Posible solapamiento de vigencia): {str(e)}")

    return {
        **request.model_dump(),
        "id": nueva_version.id,
        "hash_firma": nueva_version.hash_firma
    }

@router.post("/verify-rule/{regla_version_id}", response_model=VerifyRuleResponse)
async def verify_rule(regla_version_id: UUID, session: SessionDep):
    stmt = (
        select(ReglaVersion)
        .options(joinedload(ReglaVersion.regla))
        .where(ReglaVersion.id == regla_version_id)
    )
    result = await session.execute(stmt)
    regla = result.scalars().first()

    if not regla:
        raise HTTPException(status_code=404, detail="Rule not found")

    # Recalculate hash
    lower = regla.vigencia.lower
    upper = regla.vigencia.upper

    canonical_bytes = canonicalize_payload(
        urn_global=regla.regla.urn_global or "urn:lex:unknown",
        vigencia_desde=lower,
        vigencia_hasta=upper,
        esquema_id=str(regla.esquema_contexto_id),
        logica_json=regla.logica_json,
    )

    recalculated_hash = sha256_hash(canonical_bytes)

    return VerifyRuleResponse(
        valid=True,
        hash_match=(recalculated_hash == regla.hash_firma),
        stored_hash=regla.hash_firma or "",
        recalculated_hash=recalculated_hash,
        hash_algoritmo=regla.hash_algoritmo,
    )
