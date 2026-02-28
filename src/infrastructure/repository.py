from typing import Optional, Tuple, List
from uuid import UUID
from datetime import date
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from src.domain.models import UnidadEstructural, VersionContenido, Norma

class TemporalRepository:
    def __init__(self, session: AsyncSession):
        self.session = session

    async def get_all_normas(self) -> List[Norma]:
        """Obtiene el listado de todas las leyes/códigos en el sistema."""
        stmt = select(Norma).order_by(Norma.nombre_oficial)
        result = await self.session.execute(stmt)
        return result.scalars().all()

    async def get_unidad_by_date(self, unidad_uuid: UUID, query_date: date) -> Optional[Tuple[UnidadEstructural, VersionContenido]]:
        """Retrieves a structural unit and its content version active at the specified date."""
        stmt = (
            select(UnidadEstructural, VersionContenido)
            .join(VersionContenido, UnidadEstructural.uuid == VersionContenido.unidad_uuid)
            .where(UnidadEstructural.uuid == unidad_uuid)
            .where(VersionContenido.vigencia.contains(query_date))
        )
        result = await self.session.execute(stmt)
        return result.first()

    async def get_historial_unidad(self, unidad_uuid: UUID) -> List[VersionContenido]:
        """Obtiene todas las versiones históricas de una unidad."""
        stmt = (
            select(VersionContenido)
            .where(VersionContenido.unidad_uuid == unidad_uuid)
            # Ordenar por el límite inferior del rango de fechas
            .order_by(VersionContenido.vigencia) 
        )
        result = await self.session.execute(stmt)
        return result.scalars().all()

    async def get_estructura_norma_by_date(self, norma_id: UUID, query_date: date) -> List[Tuple[UnidadEstructural, VersionContenido]]:
        """
        Obtiene TODAS las unidades estructurales de una norma y su texto vigente 
        en una fecha específica para poder construir el árbol.
        """
        stmt = (
            select(UnidadEstructural, VersionContenido)
            .join(VersionContenido, UnidadEstructural.uuid == VersionContenido.unidad_uuid)
            .where(UnidadEstructural.norma_id == norma_id)
            .where(VersionContenido.vigencia.contains(query_date))
            .order_by(UnidadEstructural.orden_indice)
        )
        result = await self.session.execute(stmt)
        return result.all()
