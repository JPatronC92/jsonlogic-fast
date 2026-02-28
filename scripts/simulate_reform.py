import asyncio
import os
import sys
from datetime import date
from typing import Optional
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

# Add src to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from src.infrastructure.database import AsyncSessionLocal
from src.domain.models import VersionContenido
from src.core.config import get_settings
from src.domain.schemas.patch import PatchCandidate, VigenciaPatch
from src.pipeline.patcher import Patcher

settings = get_settings()

async def simulate_reform():
    print(f"Connecting to {settings.POSTGRES_DB} at {settings.POSTGRES_SERVER}...")

    fecha_reforma = date(2026, 6, 1)

    # 1. Definir el Candidato a Parche
    patch = PatchCandidate(
        decreto_fuente_url="https://dof.gob.mx/ejemplo",
        fecha_publicacion_dof=date(2026, 5, 15),
        norma_objetivo="Código Fiscal de la Federación",
        unidad_path_sugerido="Artículo 1o.",
        accion="NUEVA_VERSION",
        texto_nuevo="Texto reformado usando el Patcher: Las personas físicas y morales pagarán más impuestos.",
        nomenclatura_nueva="Artículo 1o. (Reformado con Patcher)",
        vigencia_propuesta=VigenciaPatch(inicio=fecha_reforma)
    )

    # 2. Ejecutar Parcheo
    async with AsyncSessionLocal() as session:
        patcher = Patcher(session)
        success = await patcher.aplicar_parche(patch)
        
        if not success:
            print("FAILURE: Patch application returned False.")
            return

    # 3. Verificación (Time Travel Check)
    print("\nVerifying...")
    async with AsyncSessionLocal() as session:
        # Resolvemos el UUID de nuevo para verificar
        from src.domain.services.resolver import UnidadResolver
        resolver = UnidadResolver(session)
        unit_uuid = await resolver.resolver_path(patch.norma_objetivo, patch.unidad_path_sugerido)
        
        if not unit_uuid:
            print("ERROR: Unit not found for verification.")
            return

        # Check Feb 1st (Should be Original Version)
        check_date_1 = date(2026, 2, 1)
        v1 = await get_version_at(session, unit_uuid, check_date_1)
        print(f"Query {check_date_1}: {v1.nomenclatura_visible if v1 else 'None'}")

        # Check July 1st (Should be Reformed Version)
        check_date_2 = date(2026, 7, 1)
        v2 = await get_version_at(session, unit_uuid, check_date_2)
        print(f"Query {check_date_2}: {v2.nomenclatura_visible if v2 else 'None'}")

        if v1 and v1.nomenclatura_visible == "Artículo 1o." and \
           v2 and v2.nomenclatura_visible == "Artículo 1o. (Reformado con Patcher)":
            print("\n✨ SUCCESS: Reform simulation verified via Patcher & Resolver.")
        else:
            print("\n❌ FAILURE: Verification failed.")

async def get_version_at(session: AsyncSession, unit_uuid, check_date: date) -> Optional[VersionContenido]:
    stmt = (
        select(VersionContenido)
        .where(
            VersionContenido.unidad_uuid == unit_uuid,
            VersionContenido.vigencia.contains(check_date)
        )
    )
    result = await session.execute(stmt)
    return result.scalars().first()

if __name__ == "__main__":
    if sys.platform == "win32":
        asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())

    try:
        asyncio.run(simulate_reform())
    except Exception as e:
        print(f"Error simulating reform: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
