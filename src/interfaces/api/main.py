from fastapi import FastAPI

from src.core.config import get_settings
from src.interfaces.api.routers.v1 import billing, rules

settings = get_settings()

app = FastAPI(
    title="Tempus Billing API", openapi_url=f"{settings.API_V1_STR}/openapi.json"
)

app.include_router(
    billing.router, prefix=f"{settings.API_V1_STR}/billing", tags=["billing"]
)
app.include_router(rules.router, prefix=f"{settings.API_V1_STR}/rules", tags=["rules"])


@app.get("/")
def read_root():
    return {"message": "Welcome to Tempus Billing & Commission Engine API"}
