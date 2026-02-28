import hashlib
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select
from asyncpg import Range
from src.domain.schemas.patch import PatchCandidate
from src.domain.services.resolver import UnidadResolver
from src.domain.models import VersionContenido
from src.domain.exceptions import PatchError

class Patcher:
    def __init__(self, session: AsyncSession):
        self.session = session
        self.resolver = UnidadResolver(session)

    async def aplicar_parche(self, patch: PatchCandidate) -> bool:
        """
        Toma un candidato a parche, resuelve su destino y aplica la transacción temporal.
        """
        print(f"🔧 Iniciando parcheo para: {patch.unidad_path_sugerido}")

        async with self.session.begin(): # Transacción Atómica
            # 1. Resolver UUID destino
            uuid_destino = await self.resolver.resolver_path(
                patch.norma_objetivo, 
                patch.unidad_path_sugerido
            )

            if not uuid_destino:
                if patch.accion == 'ALTA_NUEVA':
                    # TODO: Implementar lógica para crear nueva rama en el árbol
                    raise NotImplementedError("Aún no soportamos crear artículos nuevos, solo reformar.")
                else:
                    raise PatchError(f"No se encontró el artículo objetivo: {patch.unidad_path_sugerido}")

            # 2. Obtener versión vigente actual (la que tiene fecha_fin = NULL / Infinity)
            # Buscamos la versión que contiene la fecha de inicio del parche y que está abierta (upper is None)
            stmt_open = select(VersionContenido).where(
                VersionContenido.unidad_uuid == uuid_destino
            )
            result = await self.session.execute(stmt_open)
            versiones = result.scalars().all()
            
            # Encontrar la versión que actualmente es infinita
            version_anterior = next((v for v in versiones if v.vigencia.upper is None), None)

            if not version_anterior:
                print("⚠️ No hay versión vigente (infinita) para cerrar. ¿Es un artículo nuevo?")
                return False

            # 3. Cerrar vigencia anterior
            # La fecha fin es el inicio de la nueva (exclusivo)
            version_anterior.vigencia = Range(
                version_anterior.vigencia.lower, 
                patch.vigencia_propuesta.inicio
            )
            self.session.add(version_anterior) 
            
            # 4. Insertar Nueva Versión
            texto = patch.texto_nuevo or version_anterior.texto_contenido
            content_hash = hashlib.sha256(texto.encode('utf-8')).hexdigest()
            
            nueva_version = VersionContenido(
                unidad_uuid=uuid_destino,
                nomenclatura_visible=patch.nomenclatura_nueva or version_anterior.nomenclatura_visible,
                texto_contenido=texto,
                hash_contenido=content_hash,
                vigencia=Range(patch.vigencia_propuesta.inicio, None)
            )
            self.session.add(nueva_version)
        
        print(f"✅ Parche aplicado exitosamente a {uuid_destino}")
        return True
