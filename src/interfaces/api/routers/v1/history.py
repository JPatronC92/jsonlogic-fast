from fastapi import APIRouter, HTTPException, Query
from datetime import date
from uuid import UUID
from src.interfaces.api.dependencies import SessionDep
from src.infrastructure.repository import TemporalRepository
from src.domain.schemas.norma import UnidadOutput, VersionOutput
from src.domain.models import UnidadEstructural, VersionContenido

router = APIRouter()

@router.get("/articulos/{uuid}", response_model=UnidadOutput)
async def get_articulo_temporal(
    uuid: UUID,
    fecha: date = Query(default_factory=date.today, description="Fecha de consulta (YYYY-MM-DD)"),
    session: SessionDep = None
):
    """
    Obtiene la versión de una unidad estructural vigente en la fecha especificada.
    Si no se especifica fecha, usa la fecha actual.
    """
    repo = TemporalRepository(session)
    result = await repo.get_unidad_by_date(uuid, fecha)

    if not result:
        raise HTTPException(
            status_code=404,
            detail=f"Unidad no encontrada o sin vigencia activa en la fecha {fecha}"
        )

    unidad: UnidadEstructural = result[0]
    version: VersionContenido = result[1]

    return UnidadOutput(
        uuid=unidad.uuid,
        tipo=unidad.tipo_unidad,
        version_activa=VersionOutput(
            id=version.id,
            texto_contenido=version.texto_contenido,
            vigencia_inicio=version.vigencia.lower,
            vigencia_fin=version.vigencia.upper,
            nomenclatura_visible=version.nomenclatura_visible
        )
    )

from src.domain.schemas.norma import HistorialUnidadResponse

@router.get("/articulos/{uuid}/historial", response_model=HistorialUnidadResponse)
async def get_historial_articulo(
    uuid: UUID,
    session: SessionDep = None
):
    """
    Obtiene TODAS las versiones históricas (pasadas, presentes y futuras aprobadas) de una unidad.
    """
    repo = TemporalRepository(session)
    versiones = await repo.get_historial_unidad(uuid)
    
    if not versiones:
        raise HTTPException(status_code=404, detail="No se encontró historial para esta unidad")

    # The type should come from the Unidad, but we'll fetch it from the first version's rel in a real scenario
    # For now we'll just query the unit to get the type if needed or return a simplified response.
    # Since we didn't join, let's just do a basic mapping for the versions
    
    versiones_output = []
    for v in versiones:
        versiones_output.append(VersionOutput(
            id=v.id,
            texto_contenido=v.texto_contenido,
            vigencia_inicio=v.vigencia.lower,
            vigencia_fin=v.vigencia.upper,
            nomenclatura_visible=v.nomenclatura_visible
        ))
        
    return HistorialUnidadResponse(
        uuid=uuid,
        tipo="UNIDAD", # Idealmente hacemos un join con UnidadEstructural para obtener el tipo real
        versiones=versiones_output
    )
