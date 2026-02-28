import pytest
from datetime import date
from sqlalchemy.exc import IntegrityError
from asyncpg import Range
from src.domain.models import Norma, UnidadEstructural, VersionContenido
from uuid import uuid4

@pytest.mark.asyncio
async def test_overlap_constraint(session):
    """
    Smoke Test: Ensure overlapping date ranges for the same unit are rejected.
    """

    # 1. Setup base data
    norma = Norma(nombre_oficial="Ley de Prueba", estado="VIGENTE")
    session.add(norma)
    await session.flush()

    unidad = UnidadEstructural(
        norma_id=norma.id,
        tipo_unidad="ARTICULO",
        orden_indice=1.0
    )
    session.add(unidad)
    await session.flush()

    # 2. Insert Version A (Jan 1 - Feb 1)
    # Using '[]' for inclusive range or '[)' for half-open. Postgres defaults to '[)' usually.
    # DATERANGE constructor in python usually takes (lower, upper).
    # SQLAlchemy's DATERANGE might map to psycopg2.extras.DateRange or similar.
    # We can pass a string or a Range object.

    # Let's try inserting with specific dates.
    # Range: [2024-01-01, 2024-02-01)
    v1 = VersionContenido(
        unidad_uuid=unidad.uuid,
        nomenclatura_visible="Articulo 1",
        texto_contenido="Version A",
        hash_contenido="hash_b", # Fixed typo in hash
        vigencia=Range(date(2024, 1, 1), date(2024, 2, 1))
    )
    session.add(v1)
    await session.commit()

    # 3. Insert Version B (Jan 15 - Mar 1) -> Overlaps!
    v2 = VersionContenido(
        unidad_uuid=unidad.uuid,
        nomenclatura_visible="Articulo 1",
        texto_contenido="Version B",
        hash_contenido="hash_b",
        vigencia=Range(date(2024, 1, 15), date(2024, 3, 1))
    )
    session.add(v2)

    # 4. Expect Failure
    with pytest.raises(IntegrityError):
        await session.commit()

@pytest.mark.asyncio
async def test_non_overlap_constraint(session):
    """
    Ensure non-overlapping date ranges for the same unit are accepted.
    """
    norma = Norma(nombre_oficial="Ley de Prueba 2", estado="VIGENTE")
    session.add(norma)
    await session.flush()

    unidad = UnidadEstructural(
        norma_id=norma.id,
        tipo_unidad="ARTICULO",
        orden_indice=2.0
    )
    session.add(unidad)
    await session.flush()

    # Version A (Jan 1 - Feb 1)
    v1 = VersionContenido(
        unidad_uuid=unidad.uuid,
        nomenclatura_visible="Articulo 2",
        texto_contenido="Version A",
        hash_contenido="hash_a",
        vigencia=Range(date(2024, 1, 1), date(2024, 2, 1))
    )
    session.add(v1)
    await session.commit()

    # Version B (Feb 1 - Mar 1) -> No overlap since default is [)
    v2 = VersionContenido(
        unidad_uuid=unidad.uuid,
        nomenclatura_visible="Articulo 2",
        texto_contenido="Version B",
        hash_contenido="hash_b",
        vigencia=Range(date(2024, 2, 1), date(2024, 3, 1))
    )
    session.add(v2)
    
    # Expect Success
    await session.commit()
    assert v2.id is not None
