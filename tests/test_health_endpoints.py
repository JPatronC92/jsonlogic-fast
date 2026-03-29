import pytest
from httpx import ASGITransport, AsyncClient

from src.infrastructure.database import get_db
from src.interfaces.api.main import app


class HealthyDB:
    async def execute(self, query):
        return 1


@pytest.mark.asyncio
async def test_health_live():
    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://test") as client:
        response = await client.get("/health/live")

    assert response.status_code == 200
    payload = response.json()
    assert payload["status"] == "ok"
    assert payload["service"] == "tempus-api"
    assert "X-Process-Time-MS" in response.headers


@pytest.mark.asyncio
async def test_health_ready_and_health_with_db_override():
    async def override_get_db():
        yield HealthyDB()

    app.dependency_overrides[get_db] = override_get_db

    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://test") as client:
        ready_response = await client.get("/health/ready")
        health_response = await client.get("/health")

    assert ready_response.status_code == 200
    assert ready_response.json() == {"status": "ready", "database": "ok"}

    assert health_response.status_code == 200
    assert health_response.json()["status"] == "ok"
    assert health_response.json()["checks"]["database"] == "ok"

    app.dependency_overrides.pop(get_db, None)
