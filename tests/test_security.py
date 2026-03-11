import uuid
from datetime import datetime, timedelta, timezone

import jwt
import pytest
from fastapi import HTTPException

from src.core.security import (
    get_password_hash,
    ALGORITHM,
    create_access_token,
    get_current_tenant,
    get_password_hash,
    settings,
)


@pytest.mark.asyncio
async def test_get_current_tenant_valid_token():
    # Setup
    tenant_id = uuid.uuid4()
    token = create_access_token({"sub": str(tenant_id)})

    # Mock DB session
    class MockResult:
        def scalar_one_or_none(self):
            class MockTenant:
                id = tenant_id

            return MockTenant()

    class MockDB:
        async def execute(self, query):
            return MockResult()

    # Execute
    tenant = await get_current_tenant(token=token, api_key=None, db=MockDB())

    # Assert
    assert tenant.id == tenant_id


@pytest.mark.asyncio
async def test_get_current_tenant_invalid_audience():
    # Setup token with wrong audience
    tenant_id = uuid.uuid4()
    expire = datetime.now(timezone.utc) + timedelta(minutes=15)
    to_encode = {"sub": str(tenant_id), "exp": expire, "aud": "wrong-audience"}
    token = jwt.encode(to_encode, settings.SECRET_KEY, algorithm=ALGORITHM)

    # Mock DB session
    class MockDB:
        async def execute(self, query):
            pass  # Shouldn't reach here

    # Execute and Assert
    with pytest.raises(HTTPException) as exc_info:
        await get_current_tenant(token=token, api_key=None, db=MockDB())

    assert exc_info.value.status_code == 401
    assert exc_info.value.detail == "Could not validate credentials"


def test_get_password_hash():
    password = "secretpassword"
    hashed_password = get_password_hash(password)
    assert hashed_password != password
    assert len(hashed_password) > 0
    assert hashed_password.startswith("$2b$") or hashed_password.startswith("$2a$")

def test_verify_password_correct():
    password = "secretpassword"
    hashed_password = get_password_hash(password)
    assert verify_password(password, hashed_password) is True


def test_verify_password_incorrect():
    password = "secretpassword"
    hashed_password = get_password_hash(password)
    assert verify_password("wrongpassword", hashed_password) is False


def test_password_hashing_is_nondeterministic():
    password = "secretpassword"
    hash1 = get_password_hash(password)
    hash2 = get_password_hash(password)
    assert hash1 != hash2
    assert verify_password(password, hash1) is True
    assert verify_password(password, hash2) is True
