import asyncio
import uuid
from src.infrastructure.database import SessionLocal # Ajusta según tu base
from src.domain.models import Jurisdiccion, Autoridad, Dominio, AmbitoAplicacion

async def seed():
    async with SessionLocal() as session:
        # 1. Jurisdicción
        mx = Jurisdiccion(id="iso:mx", nombre="México Federal", tipo="FEDERAL")
        session.add(mx)

        # 2. Autoridad
        sat = Autoridad(id="SAT", nombre="Servicio de Administración Tributaria", jurisdiccion_id="iso:mx", sector="FINANZAS")
        session.add(sat)

        # 3. Dominio
        dom = Dominio(id="FISCAL_RENTA", descripcion="Impuesto Sobre la Renta")
        session.add(dom)

        # 4. Ámbito de Aplicación
        ambito = AmbitoAplicacion(
            id=uuid.uuid4(),
            jurisdiccion_id="iso:mx",
            autoridad_id="SAT",
            industria="FINANZAS"
        )
        session.add(ambito)

        await session.commit()
        print(f"✅ Semilla Universal plantada. Ámbito ID: {ambito.id}")

if __name__ == "__main__":
    asyncio.run(seed())
