from functools import lru_cache
from typing import Literal

from pydantic import Field, field_validator, model_validator
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    PROJECT_NAME: str = "Tempus Pricing Engine"
    API_V1_STR: str = "/api/v1"

    # Base de Datos (PostgreSQL)
    POSTGRES_USER: str
    POSTGRES_PASSWORD: str
    POSTGRES_SERVER: str
    POSTGRES_DB: str

    # Flags de Seguridad
    SECRET_KEY: str = Field(
        min_length=32,
        description="Application secret used to sign JWT tokens.",
    )
    JWT_AUDIENCE: str = Field(default="tempus-app", min_length=3)
    ENVIRONMENT: Literal["local", "staging", "production", "test"] = "local"

    @field_validator("SECRET_KEY")
    @classmethod
    def validate_secret_key(cls, value: str) -> str:
        if "super-secreto-temporal" in value.lower() or "changeme" in value.lower():
            raise ValueError(
                "SECRET_KEY uses an insecure placeholder. Generate a random secret before running."
            )
        return value

    @model_validator(mode="after")
    def validate_production_settings(self):
        if self.ENVIRONMENT == "production":
            weak_passwords = {
                "password",
                "postgres",
                "admin",
                "changeme",
                "tempus",
            }
            if self.POSTGRES_PASSWORD.lower() in weak_passwords:
                raise ValueError(
                    "POSTGRES_PASSWORD is too weak for production environment."
                )
        return self

    @property
    def SQLALCHEMY_DATABASE_URI(self) -> str:
        return (
            "postgresql+asyncpg://"
            f"{self.POSTGRES_USER}:{self.POSTGRES_PASSWORD}@{self.POSTGRES_SERVER}/{self.POSTGRES_DB}"
        )

    model_config = SettingsConfigDict(
        env_file=".env", case_sensitive=True, extra="ignore"
    )


@lru_cache
def get_settings():
    return Settings()
