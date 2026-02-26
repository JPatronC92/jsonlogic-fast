from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from uuid import UUID
from typing import Optional
from src.domain.models import Norma, UnidadEstructural, VersionContenido

class UnidadResolver:
    def __init__(self, session: AsyncSession):
        self.session = session

    async def resolver_path(self, nombre_norma: str, path_textual: str) -> Optional[UUID]:
        """
        Intenta encontrar el UUID de una unidad basándose en pistas textuales.
        
        Args:
            nombre_norma: "Código Fiscal de la Federación" (Búsqueda exacta o like)
            path_textual: "Artículo 27" o "Titulo II > Cap I > Art 27"
        
        Returns:
            UUID de la UnidadEstructural o None si no existe.
        """
        # 1. Encontrar la Norma
        stmt_norma = select(Norma).where(Norma.nombre_oficial.ilike(f"%{nombre_norma}%"))
        result_norma = await self.session.execute(stmt_norma)
        norma = result_norma.scalars().first()
        
        if not norma:
            print(f"⚠️ Norma no encontrada: {nombre_norma}")
            return None

        # 2. Extraer la parte objetivo final (ej: "Art 27" de "Titulo II > Cap I > Art 27")
        target_nomenclatura = path_textual.split(">")[-1].strip()

        # 3. Encontrar la Unidad dentro de esa Norma usando la nomenclatura objetivo
        stmt_unidad = (
            select(UnidadEstructural)
            .join(VersionContenido)
            .where(UnidadEstructural.norma_id == norma.id)
            .where(VersionContenido.nomenclatura_visible.ilike(f"%{target_nomenclatura}%"))
            .order_by(VersionContenido.created_at.desc()) # Buscar en la versión más reciente
            .limit(1)
        )
        
        result_unidad = await self.session.execute(stmt_unidad)
        unidad = result_unidad.scalars().first()
        
        if unidad:
            return unidad.uuid
        else:
            print(f"⚠️ Unidad no encontrada en {nombre_norma}: {path_textual} ({target_nomenclatura})")
            return None