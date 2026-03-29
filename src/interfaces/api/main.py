import logging
import time

from fastapi import Depends, FastAPI
from sqlalchemy import text
from sqlalchemy.ext.asyncio import AsyncSession

from src.core.config import get_settings
from src.infrastructure.database import get_db
from src.interfaces.api.routers.v1 import billing, rules

settings = get_settings()

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s %(levelname)s %(name)s %(message)s",
)
logger = logging.getLogger("tempus.api")

app = FastAPI(
    title="Tempus Billing API", openapi_url=f"{settings.API_V1_STR}/openapi.json"
)


@app.middleware("http")
async def request_timing_middleware(request, call_next):
    started_at = time.perf_counter()
    response = await call_next(request)
    duration_ms = round((time.perf_counter() - started_at) * 1000, 2)

    response.headers["X-Process-Time-MS"] = str(duration_ms)
    logger.info(
        "request_completed method=%s path=%s status_code=%s duration_ms=%s",
        request.method,
        request.url.path,
        response.status_code,
        duration_ms,
    )
    return response


app.include_router(
    billing.router, prefix=f"{settings.API_V1_STR}/billing", tags=["billing"]
)
app.include_router(rules.router, prefix=f"{settings.API_V1_STR}/rules", tags=["rules"])


@app.get("/")
def read_root():
    return {"message": "Welcome to Tempus Billing & Commission Engine API"}


@app.get("/health/live")
def health_live():
    return {"status": "ok", "service": "tempus-api"}


@app.get("/health/ready")
async def health_ready(db: AsyncSession = Depends(get_db)):
    await db.execute(text("SELECT 1"))
    return {"status": "ready", "database": "ok"}


@app.get("/health")
async def health(db: AsyncSession = Depends(get_db)):
    await db.execute(text("SELECT 1"))
    return {
        "status": "ok",
        "checks": {
            "liveness": "ok",
            "readiness": "ok",
            "database": "ok",
        },
    }
