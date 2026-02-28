import asyncio
import json
import os
import sys
import hashlib
from datetime import date
from typing import Optional, List, Dict, Any
from uuid import UUID

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from asyncpg import Range

# Add src to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from src.infrastructure.database import AsyncSessionLocal
from src.domain.models import Norma, UnidadEstructural, VersionContenido
from src.core.config import get_settings

settings = get_settings()

async def seed_cff():
    print(f"Connecting to {settings.POSTGRES_DB} at {settings.POSTGRES_SERVER}...")

    # Load JSON data
    json_path = os.path.join(os.path.dirname(__file__), "../data/cff_2026_sample.json")
    with open(json_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    async with AsyncSessionLocal() as session:
        async with session.begin():
            # 1. Create or Get Norma
            result = await session.execute(select(Norma).where(Norma.nombre_oficial == data["nombre_oficial"]))
            norma = result.scalars().first()

            if not norma:
                print(f"Creating Norma: {data['nombre_oficial']}")
                norma = Norma(
                    nombre_oficial=data["nombre_oficial"],
                    nombre_corto=data.get("nombre_corto"),
                    estado="VIGENTE" # Using string, SQLAlchemy handles Enum mapping usually if configured, or we pass the Enum member
                )
                session.add(norma)
                await session.flush() # flush to get ID
            else:
                print(f"Norma already exists: {norma.nombre_oficial}")

            # 2. Process Structure Recursively
            await process_node(session, data["estructura"], norma.id, None)

            print("Seeding complete.")

async def process_node(session: AsyncSession, nodes: List[Dict[str, Any]], norma_id: UUID, padre_id: Optional[UUID]):
    for node_data in nodes:
        # Create UnidadEstructural
        tipo_str = node_data["tipo"]
        # Map string to Enum if needed, or rely on SQLAlchemy validation
        # We'll use the string directly as it matches the Enum values defined in models

        unidad = UnidadEstructural(
            norma_id=norma_id,
            padre_id=padre_id,
            tipo_unidad=tipo_str,
            orden_indice=node_data["orden"]
        )
        session.add(unidad)
        await session.flush() # Get UUID

        print(f"  Created Unit: {node_data.get('nomenclatura', 'Unknown')} ({tipo_str}) - UUID: {unidad.uuid}")

        # Create VersionContenido (The "Golden Seed")
        # Start date: 2026-01-01, End date: None (Infinity)
        vigencia = Range(date(2026, 1, 1), None)

        # Hash content (simple placeholder for now)
        content_text = node_data.get("texto", "") or node_data.get("nomenclatura", "")
        content_hash = hashlib.sha256(content_text.encode('utf-8')).hexdigest()

        version = VersionContenido(
            unidad_uuid=unidad.uuid,
            nomenclatura_visible=node_data.get("nomenclatura", ""),
            texto_contenido=content_text,
            hash_contenido=content_hash,
            vigencia=vigencia
        )
        session.add(version)

        # Process Children
        if "hijos" in node_data and node_data["hijos"]:
            await process_node(session, node_data["hijos"], norma_id, unidad.uuid)

if __name__ == "__main__":
    if sys.platform == "win32":
        asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())

    try:
        asyncio.run(seed_cff())
    except Exception as e:
        print(f"Error seeding database: {e}")
        sys.exit(1)
