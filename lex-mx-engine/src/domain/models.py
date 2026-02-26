import uuid as uuid_pkg
from datetime import date
from typing import Optional, List
from sqlalchemy import (
    Column, String, Date, ForeignKey,
    Index, Text, Boolean, CheckConstraint, Integer, DateTime
)
from sqlalchemy.dialects.postgresql import UUID, JSONB, DATERANGE, ENUM
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column, relationship
from sqlalchemy.sql import func

# Necesario para el Exclude Constraint
from sqlalchemy.dialects.postgresql import ExcludeConstraint

class Base(DeclarativeBase):
    pass

# Enums
# We need to use create_type=False if we want to manage it purely via migrations,
# or handle it carefully. For now, we define it as part of the models.
EstadoNorma = ENUM('VIGENTE', 'ABROGADA', 'DEROGADA', name='estado_norma', create_type=False)
TipoUnidad = ENUM('TITULO', 'CAPITULO', 'ARTICULO', 'FRACCION', 'PARRAFO', name='tipo_unidad', create_type=False)
SeveridadRegla = ENUM('INFO', 'WARNING', 'ERROR', 'BLOCKER', name='severidad_regla', create_type=False)
TipoRegla = ENUM('OBLIGATORIA', 'SUGERENCIA', 'CALCULO', name='tipo_regla', create_type=False)

class Norma(Base):
    __tablename__ = "normas"

    id: Mapped[uuid_pkg.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4)
    nombre_oficial: Mapped[str] = mapped_column(String, nullable=False, index=True)
    nombre_corto: Mapped[Optional[str]] = mapped_column(String, index=True) # Ej: "CFF"
    estado = mapped_column(EstadoNorma, nullable=False, default='VIGENTE')
    metadata_norma: Mapped[dict] = mapped_column(JSONB, default={})

    # Relaciones
    unidades = relationship("UnidadEstructural", back_populates="norma", cascade="all, delete-orphan")

class UnidadEstructural(Base):
    """
    El esqueleto inmutable.
    Un artículo tiene un UUID que NO cambia aunque se renumere en el texto visible.
    """
    __tablename__ = "unidades_estructurales"

    uuid: Mapped[uuid_pkg.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4)
    norma_id: Mapped[uuid_pkg.UUID] = mapped_column(ForeignKey("normas.id"), nullable=False)
    padre_id: Mapped[Optional[uuid_pkg.UUID]] = mapped_column(ForeignKey("unidades_estructurales.uuid"), nullable=True)

    tipo_unidad = mapped_column(TipoUnidad, nullable=False)
    orden_indice: Mapped[float] = mapped_column(nullable=False) # Float para permitir inserciones (2.1)

    # Relaciones
    norma = relationship("Norma", back_populates="unidades")
    padre = relationship("UnidadEstructural", remote_side=[uuid], backref="hijos")
    versiones = relationship("VersionContenido", back_populates="unidad")

class VersionContenido(Base):
    """
    El contenido versionado temporalmente.
    Aquí vive el texto real.
    """
    __tablename__ = "versiones_contenido"

    id: Mapped[uuid_pkg.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4)
    unidad_uuid: Mapped[uuid_pkg.UUID] = mapped_column(ForeignKey("unidades_estructurales.uuid"), nullable=False)

    # Nomenclatura puede cambiar en el tiempo ("Artículo 27" -> "Artículo 28")
    nomenclatura_visible: Mapped[str] = mapped_column(String, nullable=False)
    texto_contenido: Mapped[str] = mapped_column(Text, nullable=False)
    hash_contenido: Mapped[str] = mapped_column(String(64), nullable=False) # SHA256

    # Vigencia Temporal (CRÍTICO)
    # Postgres DATERANGE: [inicio, fin)
    vigencia: Mapped[object] = mapped_column(DATERANGE, nullable=False)

    created_at: Mapped[date] = mapped_column(server_default=func.now())
    deleted_at: Mapped[Optional[date]] = mapped_column(DateTime, nullable=True)

    unidad = relationship("UnidadEstructural", back_populates="versiones")

    # CONSTRAINT DE EXCLUSIÓN (El Guardián del Tiempo)
    # Impide que dos versiones de la misma unidad se solapen en fechas.
    __table_args__ = (
        ExcludeConstraint(
            (unidad_uuid, '='),
            (vigencia, '&&'),
            name='evitar_solapamiento_temporal'
        ),
    )

class ReglaIdentidad(Base):
    """La identidad perpetua de una regla (ej: 'Tope Deducción Gasolina')."""
    __tablename__ = "reglas_identidad"
    
    uuid: Mapped[uuid_pkg.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4)
    clave_interna: Mapped[str] = mapped_column(String, unique=True, index=True) # Ej: "ISR-DED-GAS-001"
    nombre_humano: Mapped[str] = mapped_column(String)
    
    versiones = relationship("ReglaVersion", back_populates="regla", cascade="all, delete-orphan")

class ReglaVersion(Base):
    """La lógica vigente en un periodo de tiempo específico."""
    __tablename__ = "reglas_versiones"

    id: Mapped[uuid_pkg.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4)
    regla_uuid: Mapped[uuid_pkg.UUID] = mapped_column(ForeignKey("reglas_identidad.uuid"), nullable=False)
    
    # Metadatos de Ejecución
    prioridad: Mapped[int] = mapped_column(Integer, default=50) # 100 = Alta, 0 = Baja
    severidad = mapped_column(SeveridadRegla, nullable=False, default='ERROR')
    tipo = mapped_column(TipoRegla, nullable=False, default='OBLIGATORIA')
    
    # La Lógica Pura
    logica_json: Mapped[dict] = mapped_column(JSONB, nullable=False)
    contexto_schema: Mapped[Optional[dict]] = mapped_column(JSONB, nullable=True) # JSON Schema para validar input
    template_error: Mapped[str] = mapped_column(String, nullable=False) # Usaremos string.Template
    
    # Vigencia Temporal (Time Travel)
    vigencia: Mapped[object] = mapped_column(DATERANGE, nullable=False)
    
    regla = relationship("ReglaIdentidad", back_populates="versiones")

    # CONSTRAINT DE EXCLUSIÓN: Una regla no puede tener dos versiones activas al mismo tiempo
    __table_args__ = (
        ExcludeConstraint(
            (regla_uuid, '='),
            (vigencia, '&&'),
            name='evitar_solapamiento_reglas'
        ),
    )
