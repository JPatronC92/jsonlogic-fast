from fastapi import FastAPI
from src.core.config import get_settings
from src.interfaces.api.routers.v1 import compliance, history, normas

settings = get_settings()

app = FastAPI(
    title=settings.PROJECT_NAME,
    openapi_url=f"{settings.API_V1_STR}/openapi.json"
)

app.include_router(normas.router, prefix=f"{settings.API_V1_STR}/normas", tags=["normas"])
app.include_router(compliance.router, prefix=f"{settings.API_V1_STR}/compliance", tags=["compliance"])
app.include_router(history.router, prefix=f"{settings.API_V1_STR}/history", tags=["history"])

@app.get("/")
def read_root():
    return {"message": f"Welcome to {settings.PROJECT_NAME} API"}
