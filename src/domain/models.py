import uuid as uuid_pkg
from datetime import date, datetime
from typing import Optional
from sqlalchemy import (
    String, ForeignKey,
    Text, Integer, DateTime
)
from sqlalchemy.dialects.postgresql import UUID, JSONB, DATERANGE
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column, relationship
from sqlalchemy.sql import func
from sqlalchemy import event
from sqlalchemy.exc import IntegrityError
from sqlalchemy.dialects.postgresql import ExcludeConstraint

class Base(DeclarativeBase):
    pass

class PricingContextSchema(Base):
    """Schema JSON para validar el payload de la transacción antes de cobrar."""
    __tablename__ = "pricing_context_schemas"

    id: Mapped[uuid_pkg.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4)
    name: Mapped[str] = mapped_column(String, unique=True, index=True) # Ej: "Marketplace TX Payload"
    version: Mapped[int] = mapped_column(Integer, nullable=False, default=1)
    schema_json: Mapped[dict] = mapped_column(JSONB, nullable=False)

class PricingScheme(Base):
    """Agrupa múltiples reglas. Ej: 'Marketplace Standard MX', 'Enterprise VIP'"""
    __tablename__ = "pricing_schemes"

    id: Mapped[uuid_pkg.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4)
    urn: Mapped[str] = mapped_column(String, unique=True, index=True) # Ej: urn:pricing:marketplace:mx
    name: Mapped[str] = mapped_column(String)
    description: Mapped[Optional[str]] = mapped_column(String)
    
    rules = relationship("PricingRuleIdentity", back_populates="scheme", cascade="all, delete-orphan")

class PricingRuleIdentity(Base):
    """La identidad inmutable de un cargo. Ej: 'Comisión Tarjeta de Crédito'"""
    __tablename__ = "pricing_rule_identities"
    
    uuid: Mapped[uuid_pkg.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4)
    scheme_id: Mapped[uuid_pkg.UUID] = mapped_column(ForeignKey("pricing_schemes.id"), nullable=False)
    name: Mapped[str] = mapped_column(String)
    fee_type: Mapped[str] = mapped_column(String) # PERCENTAGE, FIXED_FEE, TIERED
    
    scheme = relationship("PricingScheme", back_populates="rules")
    versiones = relationship("PricingRuleVersion", back_populates="rule", cascade="all, delete-orphan")

class PricingRuleVersion(Base):
    """El corazón: La versión histórica y determinista de la regla matemática."""
    __tablename__ = "pricing_rule_versions"

    id: Mapped[uuid_pkg.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4)
    rule_uuid: Mapped[uuid_pkg.UUID] = mapped_column(ForeignKey("pricing_rule_identities.uuid"), nullable=False)
    schema_id: Mapped[uuid_pkg.UUID] = mapped_column(ForeignKey("pricing_context_schemas.id"), nullable=False)
    
    # json-logic que DEBE evaluar y retornar un NÚMERO (El monto del fee)
    logica_json: Mapped[dict] = mapped_column(JSONB, nullable=False)
    
    vigencia: Mapped[object] = mapped_column(DATERANGE, nullable=False)
    hash_firma: Mapped[Optional[str]] = mapped_column(String, nullable=True) # Trazabilidad criptográfica
    hash_algoritmo: Mapped[str] = mapped_column(String(20), nullable=False, default="SHA-256")
    
    rule = relationship("PricingRuleIdentity", back_populates="versiones")
    context_schema = relationship("PricingContextSchema")

    # Time-Travel Constraint: Imposible solapar versiones de la misma regla
    __table_args__ = (
        ExcludeConstraint(
            ('rule_uuid', '='),
            ('vigencia', '&&'),
            name='no_solapamiento_temporal_pricing'
        ),
    )

@event.listens_for(PricingRuleVersion, "before_update")
def prevent_update(mapper, connection, target):
    raise IntegrityError(
        statement=None,
        params=None,
        orig="PricingRuleVersion is immutable. Modifying historical financial rules is strictly prohibited. Create a new version instead.",
    )
