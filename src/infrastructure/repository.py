from datetime import date

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from src.domain.models import (PricingRuleIdentity, PricingRuleVersion,
                               PricingScheme)


class PricingRepository:
    def __init__(self, session: AsyncSession):
        self.session = session

    async def get_active_rules_for_scheme(
        self, scheme_urn: str, execution_date: date, tenant_id
    ) -> list[PricingRuleVersion]:
        """
        Recupera todas las versiones de reglas matemáticas (fees) asociadas a un esquema
        que estaban EXACTAMENTE activas en la fecha solicitada.
        """
        stmt = (
            select(PricingRuleVersion)
            .join(
                PricingRuleIdentity,
                PricingRuleVersion.rule_uuid == PricingRuleIdentity.uuid,
            )
            .join(PricingScheme, PricingRuleIdentity.scheme_id == PricingScheme.id)
            .options(
                selectinload(PricingRuleVersion.rule),
                selectinload(PricingRuleVersion.context_schema),
            )
            .where(
                PricingScheme.tenant_id == tenant_id,
                PricingScheme.urn == scheme_urn,
                # Magia Temporal: El operador @> de Postgres verifica si la fecha está dentro del rango
                PricingRuleVersion.vigencia.contains(execution_date),
            )
        )

        result = await self.session.execute(stmt)
        return list(result.scalars().all())

    async def get_scheme_by_urn(
        self, scheme_urn: str, tenant_id
    ) -> PricingScheme | None:
        stmt = select(PricingScheme).where(
            PricingScheme.urn == scheme_urn, PricingScheme.tenant_id == tenant_id
        )
        result = await self.session.execute(stmt)
        return result.scalars().first()
