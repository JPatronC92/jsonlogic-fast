import uuid
import pytest
from httpx import AsyncClient, ASGITransport

from src.interfaces.api.main import app
from src.core.security import get_current_tenant
from src.infrastructure.database import get_db
from src.domain.models import Tenant, PricingScheme


@pytest.mark.asyncio
async def test_list_schemes():
    # 1. Setup Data
    tenant_id = uuid.uuid4()
    tenant = Tenant(id=tenant_id, name="Test Tenant", api_keys=[])

    scheme1 = PricingScheme(
        id=uuid.uuid4(), tenant_id=tenant_id, urn="urn:scheme:test1", name="Scheme 1"
    )
    scheme2 = PricingScheme(
        id=uuid.uuid4(), tenant_id=tenant_id, urn="urn:scheme:test2", name="Scheme 2"
    )

    class MockResult:
        def scalars(self):
            class MockScalars:
                def all(self):
                    return [scheme1, scheme2]

            return MockScalars()

    class MockDB:
        async def execute(self, query):
            return MockResult()

    # 2. Setup Dependency Override for auth and db
    async def override_get_current_tenant():
        return tenant

    async def override_get_db():
        yield MockDB()

    app.dependency_overrides[get_current_tenant] = override_get_current_tenant
    app.dependency_overrides[get_db] = override_get_db

    # 3. Call endpoint
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        response = await client.get("/api/v1/rules/schemes")

    # 4. Assertions
    assert response.status_code == 200
    data = response.json()
    assert len(data) == 2

    # Sort data by name to make assertions deterministic
    data = sorted(data, key=lambda x: x["name"])

    assert data[0]["id"] == str(scheme1.id)
    assert data[0]["urn"] == "urn:scheme:test1"
    assert data[0]["name"] == "Scheme 1"

    assert data[1]["id"] == str(scheme2.id)
    assert data[1]["urn"] == "urn:scheme:test2"
    assert data[1]["name"] == "Scheme 2"

    # Cleanup dependency overrides
    app.dependency_overrides.pop(get_current_tenant, None)
    app.dependency_overrides.pop(get_db, None)


@pytest.mark.asyncio
async def test_list_schemes_empty():
    # 1. Setup Data (tenant with no schemes)
    tenant_id = uuid.uuid4()
    tenant = Tenant(id=tenant_id, name="Test Tenant Empty", api_keys=[])

    class MockResult:
        def scalars(self):
            class MockScalars:
                def all(self):
                    return []

            return MockScalars()

    class MockDB:
        async def execute(self, query):
            return MockResult()

    # 2. Setup Dependency Override for auth and db
    async def override_get_current_tenant():
        return tenant

    async def override_get_db():
        yield MockDB()

    app.dependency_overrides[get_current_tenant] = override_get_current_tenant
    app.dependency_overrides[get_db] = override_get_db

    # 3. Call endpoint
    async with AsyncClient(
        transport=ASGITransport(app=app), base_url="http://test"
    ) as client:
        response = await client.get("/api/v1/rules/schemes")

    # 4. Assertions
    assert response.status_code == 200
    data = response.json()
    assert len(data) == 0

    # Cleanup dependency overrides
    app.dependency_overrides.pop(get_current_tenant, None)
    app.dependency_overrides.pop(get_db, None)
