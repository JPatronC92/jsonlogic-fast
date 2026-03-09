"""
Tempus Engine — Database Seed Script
Generates initial Tenant, API Key, Pricing Scheme, and Rule data
for a clean-start production deployment.

Usage:
    python -m scripts.seed
"""
import asyncio
import hashlib
import secrets
import uuid
from datetime import date

from sqlalchemy.ext.asyncio import create_async_engine, async_sessionmaker, AsyncSession
from asyncpg import Range

from src.domain.models import (
    Base, Tenant, APIKey, PricingScheme,
    PricingRuleIdentity, PricingRuleVersion, PricingContextSchema
)
from src.core.config import get_settings

settings = get_settings()

engine = create_async_engine(settings.SQLALCHEMY_DATABASE_URI, echo=False)
SessionLocal = async_sessionmaker(bind=engine, class_=AsyncSession, expire_on_commit=False)


def generate_api_key() -> tuple[str, str]:
    """Generate a raw API key and its SHA-256 hash for storage."""
    raw_key = f"tk_{secrets.token_hex(32)}"
    key_hash = hashlib.sha256(raw_key.encode()).hexdigest()
    return raw_key, key_hash


async def seed():
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)

    async with SessionLocal() as session:
        # ── 1. Create Demo Tenant ──────────────────────────────
        tenant = Tenant(
            id=uuid.uuid4(),
            name="Tempus Demo Corp"
        )
        session.add(tenant)
        await session.flush()  # Get tenant.id

        # ── 2. Generate API Key ────────────────────────────────
        raw_key, key_hash = generate_api_key()
        api_key = APIKey(
            tenant_id=tenant.id,
            key_hash=key_hash,
            name="Demo Production Key",
            is_active=True
        )
        session.add(api_key)

        # ── 3. Create Context Schema ──────────────────────────
        context_schema = PricingContextSchema(
            tenant_id=tenant.id,
            name="Standard Transaction Schema",
            version=1,
            schema_json={
                "type": "object",
                "properties": {
                    "amount": {"type": "number"},
                    "method": {"type": "string"},
                    "total_volume": {"type": "number"}
                },
                "required": ["amount"]
            }
        )
        session.add(context_schema)
        await session.flush()

        # ── 4. Create Pricing Scheme ──────────────────────────
        scheme = PricingScheme(
            tenant_id=tenant.id,
            urn="urn:pricing:marketplace:mx",
            name="Marketplace MX Standard",
            description="Default pricing scheme for Mexican marketplace transactions"
        )
        session.add(scheme)
        await session.flush()

        # ── 5. Create Pricing Rule (Tiered Commission) ────────
        rule_identity = PricingRuleIdentity(
            scheme_id=scheme.id,
            name="Credit Card Commission",
            fee_type="PERCENTAGE"
        )
        session.add(rule_identity)
        await session.flush()

        # ── 6. Create Rule Version (json-logic) ──────────────
        # Tiered: >10K=1.5%, >5K=2.5%, >1K=3%, default=3.5%
        tiered_logic = {
            "if": [
                {">": [{"var": "amount"}, 10000]},
                {"*": [{"var": "amount"}, 0.015]},
                {"if": [
                    {">": [{"var": "amount"}, 5000]},
                    {"*": [{"var": "amount"}, 0.025]},
                    {"if": [
                        {">": [{"var": "amount"}, 1000]},
                        {"*": [{"var": "amount"}, 0.03]},
                        {"*": [{"var": "amount"}, 0.035]}
                    ]}
                ]}
            ]
        }

        rule_version = PricingRuleVersion(
            rule_uuid=rule_identity.uuid,
            schema_id=context_schema.id,
            logica_json=tiered_logic,
            vigencia=Range(date(2024, 1, 1), None),  # Active from Jan 2024, no end date
            hash_firma=hashlib.sha256(str(tiered_logic).encode()).hexdigest(),
            hash_algoritmo="SHA-256"
        )
        session.add(rule_version)

        await session.commit()

        # ── Print Results ─────────────────────────────────────
        print("\n" + "=" * 60)
        print("  🚀 TEMPUS ENGINE — SEED COMPLETE")
        print("=" * 60)
        print(f"\n  Tenant:       {tenant.name}")
        print(f"  Tenant ID:    {tenant.id}")
        print(f"  Scheme URN:   {scheme.urn}")
        print(f"  Rule:         {rule_identity.name} ({rule_identity.fee_type})")
        print(f"\n  🔑 API Key (SAVE THIS — shown only once):")
        print(f"     {raw_key}")
        print(f"\n  Use it with:")
        print(f'     curl -H "X-API-Key: {raw_key}" ...')
        print("=" * 60 + "\n")


if __name__ == "__main__":
    asyncio.run(seed())
