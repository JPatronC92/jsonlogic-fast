import asyncio
import os
import sys
from datetime import date
from psycopg2.extras import DateRange

# Add src to the Python path
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))

from src.infrastructure.database import AsyncSessionLocal
from src.domain.models import ReglaIdentidad, ReglaVersion

async def seed_universal_rule():
    async with AsyncSessionLocal() as session:
        print("🌱 Seeding Universal Rule for Microchips (Aduana MEX)...")
        
        # 1. Crear Identidad de la Regla
        regla_aduana = ReglaIdentidad(
            clave_interna="ADUANA-MEX-8542-NOM",
            nombre_humano="Regla de Certificación NOM para Microchips"
        )
        session.add(regla_aduana)
        await session.flush()  # Para obtener el UUID

        # 2. Crear Versión de la Regla (Vigente en 2026)
        version_2026 = ReglaVersion(
            regla_uuid=regla_aduana.uuid,
            prioridad=100,
            severidad="BLOCKER",
            tipo="OBLIGATORIA",
            template_error="El embarque de ${origen} requiere Certificado NOM-019 porque su valor ($${valor_usd}) supera los $1000 USD.",
            logica_json={
                "if": [
                    {
                        "and": [
                            {"==": [{"var": "codigo_hs"}, "8542.31"]},
                            {">": [{"var": "valor_usd"}, 1000]}
                        ]
                    },
                    {
                        "or": [
                            {"==": [{"var": "tiene_certificado_nom"}, True]},
                            {"in": [{"var": "origen"}, ["USA", "CAN"]]}
                        ]
                    },
                    True
                ]
            },
            # Vigente desde enero 2026 hasta indefinido (upper=None)
            vigencia=DateRange(date(2026, 1, 1), None)
        )
        session.add(version_2026)
        
        await session.commit()
        print("✅ Universal Rule seeded successfully!")

if __name__ == "__main__":
    asyncio.run(seed_universal_rule())
