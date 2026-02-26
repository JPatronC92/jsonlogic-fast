from sqlalchemy.ext.asyncio import create_async_engine, async_sessionmaker, AsyncSession
from src.core.config import get_settings

settings = get_settings()

engine = create_async_engine(
    settings.SQLALCHEMY_DATABASE_URI,
    echo=True, # Set to False in production
    future=True
)

AsyncSessionLocal = async_sessionmaker(
    bind=engine,
    class_=AsyncSession,
    expire_on_commit=False,
    autoflush=False
)

async def get_db() -> AsyncSession:
    async with AsyncSessionLocal() as session:
        yield session
