import uuid

import pytest
from fastapi import HTTPException

from src.domain.models import PricingScheme, Tenant
from src.interfaces.api.routers.v1.rules import PricingSchemeCreate, create_scheme


from unittest.mock import AsyncMock, MagicMock

@pytest.fixture
def mock_session():
    session = AsyncMock()
    # add is synchronous in SQLAlchemy
    session.add = MagicMock()
    return session

@pytest.fixture
def dummy_tenant():
    return Tenant(id=uuid.uuid4(), name=f"test_tenant_{uuid.uuid4().hex[:6]}")


@pytest.mark.asyncio
async def test_create_scheme_success(mock_session, dummy_tenant):
    request = PricingSchemeCreate(
        urn="urn:pricing:test:standard",
        name="Test Standard Scheme",
        description="A test scheme",
    )

    # Mocking session.execute to return no results for duplicate URN check
    mock_result = MagicMock()
    mock_result.scalars().first.return_value = None
    mock_session.execute.return_value = mock_result

    scheme = await create_scheme(request=request, session=mock_session, tenant=dummy_tenant)

    assert scheme is not None
    assert scheme.tenant_id == dummy_tenant.id
    assert scheme.urn == request.urn
    assert scheme.name == request.name
    assert scheme.description == request.description

    # Verify session methods were called
    mock_session.add.assert_called_once()
    mock_session.commit.assert_awaited_once()
    mock_session.refresh.assert_awaited_once_with(scheme)


@pytest.mark.asyncio
async def test_create_scheme_duplicate_urn(mock_session, dummy_tenant):
    urn = "urn:pricing:test:duplicate"
    request = PricingSchemeCreate(
        urn=urn,
        name="Scheme Duplicate",
    )

    # Mocking session.execute to return an existing scheme
    existing_scheme = PricingScheme(tenant_id=dummy_tenant.id, urn=urn, name="Existing")
    mock_result = MagicMock()
    mock_result.scalars().first.return_value = existing_scheme
    mock_session.execute.return_value = mock_result

    with pytest.raises(HTTPException) as exc_info:
        await create_scheme(request=request, session=mock_session, tenant=dummy_tenant)

    assert exc_info.value.status_code == 400
    assert exc_info.value.detail == "A scheme with this URN already exists."
    mock_session.add.assert_not_called()
