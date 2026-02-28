import asyncio
import os
import sys
from datetime import date
from sqlalchemy.ext.asyncio import create_async_engine, async_sessionmaker
from asyncpg import Range

sys.path.append(os.path.join(os.path.dirname(__file__), '..'))

from src.core.config import get_settings
from src.domain.models import Base, PricingScheme, PricingRuleIdentity, PricingRuleVersion, PricingContextSchema

settings = get_settings()
engine = create_async_engine(settings.SQLALCHEMY_DATABASE_URI)
AsyncSessionLocal = async_sessionmaker(engine, expire_on_commit=False)

async def seed():
    async with engine.begin() as conn:
        print("0. Limpiando base de datos (Eliminando tablas antiguas)...")
        await conn.run_sync(Base.metadata.drop_all)
        print("1. Creando tablas en DB...")
        await conn.run_sync(Base.metadata.create_all)
        
    async with AsyncSessionLocal() as session:
        print("2. Creando JSON Schema para validar transacciones de pago...")
        schema = PricingContextSchema(
            name="Stripe-like Payment Payload",
            version=1,
            schema_json={
                "type": "object",
                "properties": {
                    "amount": {"type": "number"},
                    "currency": {"type": "string"},
                    "payment_method": {"enum": ["CREDIT_CARD", "DEBIT_CARD", "OXXO_PAY"]},
                    "country": {"type": "string"}
                },
                "required": ["amount", "currency", "payment_method", "country"]
            }
        )
        session.add(schema)
        await session.flush()

        print("3. Creando Esquema de Precios (Stripe MX)...")
        scheme = PricingScheme(
            urn="urn:pricing:stripe:mx:standard",
            name="Tarifas Estándar México",
            description="Tarifas base para pagos procesados en México."
        )
        session.add(scheme)
        await session.flush()

        print("4. Creando Regla: Comisión Porcentual (3.6%)...")
        rule_percent = PricingRuleIdentity(
            scheme_id=scheme.id,
            name="Comisión Base Procesamiento",
            fee_type="PERCENTAGE"
        )
        session.add(rule_percent)
        await session.flush()

        # Versión 1 de la regla (Vigente desde Ene 2024 hasta el infinito)
        # logica_json: if country == "MX" then amount * 0.036 else 0
        v1_percent = PricingRuleVersion(
            rule_uuid=rule_percent.uuid,
            schema_id=schema.id,
            logica_json={
                "if": [
                    {"==": [{"var": "country"}, "MX"]},
                    {"*": [{"var": "amount"}, 0.036]},
                    0
                ]
            },
            vigencia=Range(date(2024, 1, 1), None, lower_inc=True, upper_inc=False),
            hash_firma="sha256:initial_seed_hash_001"
        )
        session.add(v1_percent)

        print("5. Creando Regla: Tarifa Fija por Transacción ($3.00 MXN)...")
        rule_fixed = PricingRuleIdentity(
            scheme_id=scheme.id,
            name="Fee Fijo OXXO/Tarjeta",
            fee_type="FIXED_FEE"
        )
        session.add(rule_fixed)
        await session.flush()

        v1_fixed = PricingRuleVersion(
            rule_uuid=rule_fixed.uuid,
            schema_id=schema.id,
            # Lógica: $3.00 flat fee (sin importar el monto)
            logica_json=3.00, 
            vigencia=Range(date(2024, 1, 1), None, lower_inc=True, upper_inc=False),
            hash_firma="sha256:initial_seed_hash_002"
        )
        session.add(v1_fixed)

        await session.commit()
        print("✅ Base de datos sembrada con éxito. Listo para calcular comisiones.")

if __name__ == "__main__":
    asyncio.run(seed())
