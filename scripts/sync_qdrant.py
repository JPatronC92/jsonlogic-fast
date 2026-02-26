import asyncio
import os
import sys
from datetime import date
from sqlalchemy import select
from sqlalchemy.orm import selectinload

# Add src to the Python path
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))

from src.infrastructure.database import AsyncSessionLocal
from src.domain.models import VersionContenido, UnidadEstructural, Norma
from src.infrastructure.vector_store import VectorStore

async def sync_database_to_qdrant():
    print("🚀 Iniciando sincronización Semántica: PostgreSQL -> Qdrant")
    
    # 1. Inicializar Vector Store
    vector_store = VectorStore()
    await vector_store.initialize_collection()
    
    async with AsyncSessionLocal() as session:
        # 2. Extraer el conocimiento de PostgreSQL (Versiones de Contenido)
        # Cargamos la unidad y la norma para tener contexto rico
        stmt = (
            select(VersionContenido)
            .options(
                selectinload(VersionContenido.unidad).selectinload(UnidadEstructural.norma)
            )
        )
        result = await session.execute(stmt)
        versiones = result.scalars().all()
        
        if not versiones:
            print("⚠️ No hay versiones de contenido en la base de datos para vectorizar.")
            print("   Asegúrate de haber corrido los seeders (ej. python scripts/seed_cff_2026.py).")
            return

        print(f"📚 Encontradas {len(versiones)} versiones de contenido. Vectorizando...")
        
        # 3. Vectorizar e Inyectar en Qdrant
        for i, version in enumerate(versiones):
            unidad = version.unidad
            norma = unidad.norma
            
            # Crear un texto enriquecido para mejorar el contexto del LLM
            texto_enriquecido = f"Ley: {norma.nombre_corto or norma.nombre_oficial}
"
            texto_enriquecido += f"Sección: {version.nomenclatura_visible}
"
            texto_enriquecido += f"Contenido: {version.texto_contenido}"
            
            inicio = version.vigencia.lower
            fin = version.vigencia.upper
            
            metadata = {
                "norma": norma.nombre_corto or norma.nombre_oficial,
                "nomenclatura": version.nomenclatura_visible
            }
            
            try:
                await vector_store.upsert_version(
                    version_id=version.id,
                    unidad_id=version.unidad_uuid,
                    texto=texto_enriquecido,
                    vigencia_inicio=inicio,
                    vigencia_fin=fin,
                    metadata=metadata
                )
                print(f"  [{i+1}/{len(versiones)}] ✅ Vectorizado: {norma.nombre_corto} - {version.nomenclatura_visible}")
            except Exception as e:
                 print(f"  [{i+1}/{len(versiones)}] ❌ Error vectorizando {version.id}: {e}")

        print("✨ Sincronización completada exitosamente.")
        
        # 4. Prueba rápida de búsqueda semántica con viaje en el tiempo
        print("
🔍 Probando el RAG Legal Definitivo...")
        query = "¿Qué impuestos aplican a la importación o cuáles son los requisitos aduanales?"
        fecha_prueba = date(2026, 3, 15)
        
        print(f"   Consulta: '{query}'")
        print(f"   Máquina del Tiempo ajustada a: {fecha_prueba}")
        
        try:
            resultados = await vector_store.semantic_search(query=query, fecha_operacion=fecha_prueba, limit=2)
            if resultados:
                for r in resultados:
                    print(f"   -> [Score: {r['score']:.4f}] {r['texto'][:100]}...")
            else:
                 print("   -> No se encontraron resultados relevantes vigentes en esa fecha.")
        except Exception as e:
             print(f"   ❌ Error en la búsqueda de prueba: {e}")

if __name__ == "__main__":
    asyncio.run(sync_database_to_qdrant())