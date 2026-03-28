import pytest
from pydantic import ValidationError

from src.core.config import Settings


def _base_kwargs():
    return {
        "POSTGRES_USER": "postgres",
        "POSTGRES_PASSWORD": "strong-pass-123",
        "POSTGRES_SERVER": "localhost",
        "POSTGRES_DB": "tempus_db",
        "SECRET_KEY": "a" * 40,
        "JWT_AUDIENCE": "tempus-app",
        "ENVIRONMENT": "local",
    }


def test_settings_accept_valid_values():
    settings = Settings(**_base_kwargs())
    assert settings.ENVIRONMENT == "local"
    assert settings.JWT_AUDIENCE == "tempus-app"


def test_settings_reject_placeholder_secret_key():
    kwargs = _base_kwargs()
    kwargs["SECRET_KEY"] = "super-secreto-temporal-que-no-deberia-usarse"

    with pytest.raises(ValidationError):
        Settings(**kwargs)


def test_settings_reject_weak_db_password_in_production():
    kwargs = _base_kwargs()
    kwargs["ENVIRONMENT"] = "production"
    kwargs["POSTGRES_PASSWORD"] = "password"

    with pytest.raises(ValidationError):
        Settings(**kwargs)
