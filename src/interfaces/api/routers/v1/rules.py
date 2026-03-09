from fastapi import APIRouter, HTTPException, Depends
from pydantic import BaseModel
from typing import Dict, Any, List, Optional
from uuid import UUID
from datetime import date
from sqlalchemy import select

from src.interfaces.api.dependencies import SessionDep
from src.core.security import get_current_tenant
from src.domain.models import (
    Tenant,
    PricingScheme,
    PricingRuleIdentity,
    PricingRuleVersion,
)

router = APIRouter()


class PricingSchemeCreate(BaseModel):
    urn: str
    name: str
    description: Optional[str] = None


class PricingSchemeResponse(BaseModel):
    id: UUID
    urn: str
    name: str

    class Config:
        from_attributes = True


class RuleVersionCreate(BaseModel):
    rule_name: str
    fee_type: str
    schema_id: UUID
    logica_json: Dict[str, Any]
    vigencia_start: date
    vigencia_end: Optional[date] = None


@router.get("/schemes", response_model=List[PricingSchemeResponse])
async def list_schemes(
    session: SessionDep, tenant: Tenant = Depends(get_current_tenant)
):
    """List all pricing schemes for the current tenant."""
    result = await session.execute(
        select(PricingScheme).where(PricingScheme.tenant_id == tenant.id)
    )
    return result.scalars().all()


@router.post("/schemes", response_model=PricingSchemeResponse)
async def create_scheme(
    request: PricingSchemeCreate,
    session: SessionDep,
    tenant: Tenant = Depends(get_current_tenant),
):
    """Create a new pricing scheme (e.g., standard, premium tier)."""
    # Verify URN is unique for the tenant
    result = await session.execute(
        select(PricingScheme).where(
            PricingScheme.urn == request.urn, PricingScheme.tenant_id == tenant.id
        )
    )
    if result.scalars().first():
        raise HTTPException(
            status_code=400, detail="A scheme with this URN already exists."
        )

    scheme = PricingScheme(
        tenant_id=tenant.id,
        urn=request.urn,
        name=request.name,
        description=request.description,
    )
    session.add(scheme)
    await session.commit()
    await session.refresh(scheme)
    return scheme


@router.post("/schemes/{scheme_id}/rules")
async def add_rule_version(
    scheme_id: UUID,
    request: RuleVersionCreate,
    session: SessionDep,
    tenant: Tenant = Depends(get_current_tenant),
):
    """Adds a new deterministic math rule version to a scheme."""

    # 1. Validate scheme belongs to tenant
    result = await session.execute(
        select(PricingScheme).where(
            PricingScheme.id == scheme_id, PricingScheme.tenant_id == tenant.id
        )
    )
    scheme = result.scalars().first()
    if not scheme:
        raise HTTPException(status_code=404, detail="Pricing Scheme not found")

    # 2. Add the Rule Identity (in a real app, we check if it exists first by name)
    rule_identity = PricingRuleIdentity(
        scheme_id=scheme.id, name=request.rule_name, fee_type=request.fee_type
    )
    session.add(rule_identity)
    await session.flush()  # Get the UUID

    # 3. Add the Rule Version
    # asyncpg handles ranges natively when passed as strings.
    # To be safe across drivers:

    # For simplicity, we assume the vigencia starts today and goes forward forever.
    vigencia_range = f"[{request.vigencia_start.isoformat()},"
    if request.vigencia_end:
        vigencia_range += f"{request.vigencia_end.isoformat()})"
    else:
        vigencia_range += ")"

    # We will use raw SQL for insertion if necessary, but DATERANGE mapped column often accepts strings.
    rule_version = PricingRuleVersion(
        rule_uuid=rule_identity.uuid,
        schema_id=request.schema_id,
        logica_json=request.logica_json,
        vigencia=vigencia_range,
    )
    session.add(rule_version)
    await session.commit()

    return {"message": "Rule successfully created.", "rule_version": rule_version.id}
