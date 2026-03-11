import uuid
from datetime import datetime, timedelta, timezone
from typing import Optional

import jwt
from passlib.context import CryptContext
from fastapi import Depends, HTTPException, status
from fastapi.security import OAuth2PasswordBearer, APIKeyHeader
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select
from sqlalchemy.orm import selectinload

from src.core.config import get_settings
from src.infrastructure.database import get_db
from src.domain.models import Tenant, APIKey

settings = get_settings()

pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")

# Para el dashboard y UI
oauth2_scheme = OAuth2PasswordBearer(
    tokenUrl=f"{settings.API_V1_STR}/auth/login", auto_error=False
)

# Para SDKs y B2B
api_key_header = APIKeyHeader(name="X-API-Key", auto_error=False)

ALGORITHM = "HS256"
# In a real app, this should be longer or have refresh tokens
ACCESS_TOKEN_EXPIRE_MINUTES = 60 * 24 * 7


def verify_password(plain_password, hashed_password):
    return pwd_context.verify(plain_password, hashed_password)


def get_password_hash(password):
    return pwd_context.hash(password)


def create_access_token(data: dict, expires_delta: Optional[timedelta] = None):
    to_encode = data.copy()
    if expires_delta:
        expire = datetime.now(timezone.utc) + expires_delta
    else:
        expire = datetime.now(timezone.utc) + timedelta(
            minutes=ACCESS_TOKEN_EXPIRE_MINUTES
        )
    to_encode.update({"exp": expire, "aud": settings.JWT_AUDIENCE})
    encoded_jwt = jwt.encode(to_encode, settings.SECRET_KEY, algorithm=ALGORITHM)
    return encoded_jwt


async def get_tenant_by_api_key(api_key: str, db: AsyncSession) -> Optional[Tenant]:
    """Retrieve the tenant associated with a given API Key hash."""
    # Assuming the API Key passed is already hashed or we do a simple query
    # In a prod environment, the raw API key should be hashed. For MVP we'll query directly
    # Actually, passlib's verify is used for bcrypt because the hash changes.
    # Usually API keys are SHA-256 hashed once if we want to query them quickly.
    # To keep it simple, we'll assume `key_hash` is just the plain key for MVP or we query and verify.
    # We will refine this later if needed. For now, we search by `key_hash` assuming a fast hash like SHA256 is used.
    # If the user wants bcrypt, we'd have to load all API keys, which is slow.
    # Let's assume `key_hash` is a simple SHA256 hex digest for API Keys.
    import hashlib

    sha256_hash = hashlib.sha256(api_key.encode()).hexdigest()

    result = await db.execute(
        select(APIKey)
        .options(selectinload(APIKey.tenant))
        .where(APIKey.key_hash == sha256_hash, APIKey.is_active == True)  # noqa: E712
    )
    api_key_obj = result.scalar_one_or_none()
    if api_key_obj:
        return api_key_obj.tenant
    return None


async def get_tenant_by_jwt(token: str, db: AsyncSession) -> Optional[Tenant]:
    """Retrieve the tenant associated with a given JWT token."""
    try:
        payload = jwt.decode(
            token,
            settings.SECRET_KEY,
            algorithms=[ALGORITHM],
            audience=settings.JWT_AUDIENCE,
        )
        tenant_id_str: str = payload.get("sub")
        if tenant_id_str is None:
            return None

        tenant_id = uuid.UUID(tenant_id_str)
    except (jwt.PyJWTError, ValueError):
        return None

    result = await db.execute(select(Tenant).where(Tenant.id == tenant_id))
    return result.scalar_one_or_none()


async def get_current_tenant(
    token: str = Depends(oauth2_scheme),
    api_key: str = Depends(api_key_header),
    db: AsyncSession = Depends(get_db),
) -> Tenant:
    """
    Dependency that resolves the active Tenant.
    It checks for an API Key first (B2B/SDK), then falls back to JWT (Dashboard).
    """
    credentials_exception = HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail="Could not validate credentials",
        headers={"WWW-Authenticate": "Bearer"},
    )

    tenant = None

    if api_key:
        tenant = await get_tenant_by_api_key(api_key, db)
        if not tenant:
            raise credentials_exception
        return tenant

    if token:
        tenant = await get_tenant_by_jwt(token, db)
        if tenant is None:
            raise credentials_exception
        return tenant

    raise HTTPException(
        status_code=status.HTTP_401_UNAUTHORIZED,
        detail="Not authenticated",
        headers={"WWW-Authenticate": "Bearer"},
    )
