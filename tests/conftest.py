import pytest
from typing import AsyncGenerator
from sqlalchemy.ext.asyncio import create_async_engine, AsyncSession, async_sessionmaker
from src.core.config import get_settings
from src.domain.models import Base

settings = get_settings()


@pytest.fixture(scope="function")
async def engine():
    engine = create_async_engine(settings.SQLALCHEMY_DATABASE_URI, future=True)
    try:
        # Check connection
        async with engine.connect():
            pass
        yield engine
    except Exception as e:
        pytest.skip(f"Database not available: {e}")
    finally:
        await engine.dispose()


@pytest.fixture(scope="function")
async def session(engine) -> AsyncGenerator[AsyncSession, None]:
    # Create tables for testing
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)

    SessionLocal = async_sessionmaker(
        bind=engine, class_=AsyncSession, expire_on_commit=False
    )
    async with SessionLocal() as session:
        yield session
        await session.rollback()

    # Cleanup
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.drop_all)
