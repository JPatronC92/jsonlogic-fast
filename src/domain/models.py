import uuid as uuid_pkg
from datetime import datetime
from typing import Optional

from sqlalchemy import Boolean, DateTime, ForeignKey, Integer, String, event
from sqlalchemy.dialects.postgresql import (DATERANGE, JSONB, UUID,
                                            ExcludeConstraint)
from sqlalchemy.exc import IntegrityError
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column, relationship
from sqlalchemy.sql import func


class Base(DeclarativeBase):
    pass


class Tenant(Base):
    """Organización o empresa dueña de las reglas de pricing."""

    __tablename__ = "tenants"

    id: Mapped[uuid_pkg.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4
    )
    name: Mapped[str] = mapped_column(String, unique=True, index=True)
    created_at: Mapped[datetime] = mapped_column(DateTime, default=func.now())

    api_keys = relationship(
        "APIKey", back_populates="tenant", cascade="all, delete-orphan"
    )
    schemes = relationship(
        "PricingScheme", back_populates="tenant", cascade="all, delete-orphan"
    )
    context_schemas = relationship(
        "PricingContextSchema", back_populates="tenant", cascade="all, delete-orphan"
    )


class APIKey(Base):
    """Clave de acceso para la integración B2B vía SDK."""

    __tablename__ = "api_keys"

    id: Mapped[uuid_pkg.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4
    )
    tenant_id: Mapped[uuid_pkg.UUID] = mapped_column(
        ForeignKey("tenants.id"), nullable=False
    )
    key_hash: Mapped[str] = mapped_column(
        String, unique=True, index=True, nullable=False
    )
    name: Mapped[str] = mapped_column(String, nullable=False)
    created_at: Mapped[datetime] = mapped_column(DateTime, default=func.now())
    is_active: Mapped[bool] = mapped_column(Boolean, default=True)

    tenant = relationship("Tenant", back_populates="api_keys")


class PricingContextSchema(Base):
    """Schema JSON para validar el payload de la transacción antes de cobrar."""

    __tablename__ = "pricing_context_schemas"

    id: Mapped[uuid_pkg.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4
    )
    tenant_id: Mapped[uuid_pkg.UUID] = mapped_column(
        ForeignKey("tenants.id"), nullable=False
    )
    name: Mapped[str] = mapped_column(
        String, index=True
    )  # Removed unique=True because names can repeat across tenants
    version: Mapped[int] = mapped_column(Integer, nullable=False, default=1)
    schema_json: Mapped[dict] = mapped_column(JSONB, nullable=False)

    tenant = relationship("Tenant", back_populates="context_schemas")


class PricingScheme(Base):
    """Agrupa múltiples reglas. Ej: 'Marketplace Standard MX', 'Enterprise VIP'"""

    __tablename__ = "pricing_schemes"

    id: Mapped[uuid_pkg.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4
    )
    tenant_id: Mapped[uuid_pkg.UUID] = mapped_column(
        ForeignKey("tenants.id"), nullable=False
    )
    urn: Mapped[str] = mapped_column(
        String, unique=True, index=True
    )  # Ej: urn:pricing:marketplace:mx
    name: Mapped[str] = mapped_column(String)
    description: Mapped[Optional[str]] = mapped_column(String)

    tenant = relationship("Tenant", back_populates="schemes")
    rules = relationship(
        "PricingRuleIdentity", back_populates="scheme", cascade="all, delete-orphan"
    )


class PricingRuleIdentity(Base):
    """La identidad inmutable de un cargo. Ej: 'Comisión Tarjeta de Crédito'"""

    __tablename__ = "pricing_rule_identities"

    uuid: Mapped[uuid_pkg.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4
    )
    scheme_id: Mapped[uuid_pkg.UUID] = mapped_column(
        ForeignKey("pricing_schemes.id"), nullable=False
    )
    name: Mapped[str] = mapped_column(String)
    fee_type: Mapped[str] = mapped_column(String)  # PERCENTAGE, FIXED_FEE, TIERED

    scheme = relationship("PricingScheme", back_populates="rules")
    versiones = relationship(
        "PricingRuleVersion", back_populates="rule", cascade="all, delete-orphan"
    )


class PricingRuleVersion(Base):
    """El corazón: La versión histórica y determinista de la regla matemática."""

    __tablename__ = "pricing_rule_versions"

    id: Mapped[uuid_pkg.UUID] = mapped_column(
        UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4
    )
    rule_uuid: Mapped[uuid_pkg.UUID] = mapped_column(
        ForeignKey("pricing_rule_identities.uuid"), nullable=False
    )
    schema_id: Mapped[uuid_pkg.UUID] = mapped_column(
        ForeignKey("pricing_context_schemas.id"), nullable=False
    )

    # json-logic que DEBE evaluar y retornar un NÚMERO (El monto del fee)
    logica_json: Mapped[dict] = mapped_column(JSONB, nullable=False)

    vigencia: Mapped[object] = mapped_column(DATERANGE, nullable=False)
    hash_firma: Mapped[Optional[str]] = mapped_column(
        String, nullable=True
    )  # Trazabilidad criptográfica
    hash_algoritmo: Mapped[str] = mapped_column(
        String(20), nullable=False, default="SHA-256"
    )

    rule = relationship("PricingRuleIdentity", back_populates="versiones")
    context_schema = relationship("PricingContextSchema")

    # Mapped for internal JSON string cache
    __allow_unmapped__ = True
    _logica_json_str: Optional[str] = None
    _validator: Optional[object] = None

    @property
    def validator(self):
        if self._validator is None:
            from jsonschema import Draft7Validator

            self._validator = Draft7Validator(self.context_schema.schema_json)
        return self._validator

    @property
    def logica_json_str(self) -> str:
        """Cachea la representación en string del json-logic para optimización."""
        if self._logica_json_str is None:
            import json

            self._logica_json_str = json.dumps(self.logica_json)
        return self._logica_json_str

    # Time-Travel Constraint: Imposible solapar versiones de la misma regla
    __table_args__ = (
        ExcludeConstraint(
            ("rule_uuid", "="),
            ("vigencia", "&&"),
            name="no_solapamiento_temporal_pricing",
        ),
    )


@event.listens_for(PricingRuleVersion, "before_update")
def prevent_update(mapper, connection, target):
    raise IntegrityError(
        statement=None,
        params=None,
        orig="PricingRuleVersion is immutable. Modifying historical financial rules is strictly prohibited. Create a new version instead.",
    )
