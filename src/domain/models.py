import uuid as uuid_pkg
from datetime import date, datetime
from typing import Optional
from sqlalchemy import (
    String, ForeignKey,
    Text, Integer, DateTime
)
from sqlalchemy.dialects.postgresql import UUID, JSONB, DATERANGE, ENUM
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column, relationship
from sqlalchemy.sql import func
from sqlalchemy import event
from sqlalchemy.exc import IntegrityError

# Necesario para el Exclude Constraint
from sqlalchemy.dialects.postgresql import ExcludeConstraint

class Base(DeclarativeBase):
    pass

# Enums
# We need to use create_type=False if we want to manage it purely via migrations,
# or handle it carefully. For now, we define it as part of the models.
# Changing to create_type=True so tests using Base.metadata.create_all() can automatically create the ENUMs
EstadoNorma = ENUM('VIGENTE', 'ABROGADA', 'DEROGADA', name='estado_norma', create_type=True)
TipoUnidad = ENUM('TITULO', 'CAPITULO', 'ARTICULO', 'FRACCION', 'PARRAFO', name='tipo_unidad', create_type=True)
SeveridadRegla = ENUM('INFO', 'WARNING', 'ERROR', 'BLOCKER', name='severidad_regla', create_type=True)
TipoRegla = ENUM('OBLIGATORIA', 'SUGERENCIA', 'CALCULO', name='tipo_regla', create_type=True)

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

# --- CAPA 1: Clasificación Geopolítica ---

class Jurisdiccion(Base):
    __tablename__ = "jurisdicciones"

    id: Mapped[str] = mapped_column(String, primary_key=True) # Ej: "iso:mx"
    nombre: Mapped[str] = mapped_column(String, nullable=False)
    tipo: Mapped[str] = mapped_column(String) # FEDERAL, ESTATAL, SUPRANACIONAL
    parent_id: Mapped[Optional[str]] = mapped_column(String, ForeignKey("jurisdicciones.id"), nullable=True)

class Autoridad(Base):
    __tablename__ = "autoridades"

    id: Mapped[str] = mapped_column(String, primary_key=True) # Ej: "SAT"
    nombre: Mapped[str] = mapped_column(String, nullable=False)
    jurisdiccion_id: Mapped[str] = mapped_column(String, ForeignKey("jurisdicciones.id"))
    sector: Mapped[str] = mapped_column(String) # SALUD, FINANZAS, LOGISTICA

# --- CAPA 2: Dominio Abstracto ---

class Dominio(Base):
    __tablename__ = "dominios"

    id: Mapped[str] = mapped_column(String, primary_key=True) # Ej: "FISCAL_RENTA"
    descripcion: Mapped[str] = mapped_column(String)

# --- CAPA 4: Ámbito de Aplicación ---

class AmbitoAplicacion(Base):
    __tablename__ = "ambitos_aplicacion"

    id: Mapped[uuid_pkg.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4)
    jurisdiccion_id: Mapped[str] = mapped_column(String, ForeignKey("jurisdicciones.id"))
    autoridad_id: Mapped[str] = mapped_column(String, ForeignKey("autoridades.id"))
    industria: Mapped[str] = mapped_column(String)

# --- CAPA 6: Versionado de Esquema (Contexto) ---

class EsquemaContexto(Base):
    __tablename__ = "esquemas_contexto"

    id: Mapped[uuid_pkg.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4)
    dominio_id: Mapped[str] = mapped_column(String, ForeignKey("dominios.id"))
    version: Mapped[int] = mapped_column(Integer, nullable=False)
    schema_json: Mapped[dict] = mapped_column(JSONB, nullable=False) # JSON Schema formal

# --- CAPA 3: Identidad Universal ---

class ReglaIdentidad(Base):
    __tablename__ = "reglas_identidad"
    
    uuid: Mapped[uuid_pkg.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4)
    urn_global: Mapped[Optional[str]] = mapped_column(String, unique=True, index=True, nullable=True) # Nuevo estándar
    clave_interna: Mapped[Optional[str]] = mapped_column(String, index=True, nullable=True) # Retrocompatibilidad

    dominio_id: Mapped[str] = mapped_column(String, ForeignKey("dominios.id"))
    tipo_obligacion: Mapped[str] = mapped_column(String) # LIMITE, PROHIBICION, REQUISITO
    criticidad: Mapped[str] = mapped_column(String) # INFO, WARNING, BLOCKER
    
    versiones = relationship("ReglaVersion", back_populates="regla", cascade="all, delete-orphan")

# --- CAPA 5: Implementación Temporal (El Núcleo) ---

class ReglaVersion(Base):
    __tablename__ = "reglas_versiones"

    id: Mapped[uuid_pkg.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid_pkg.uuid4)
    regla_uuid: Mapped[uuid_pkg.UUID] = mapped_column(ForeignKey("reglas_identidad.uuid"), nullable=False)
    ambito_id: Mapped[uuid_pkg.UUID] = mapped_column(ForeignKey("ambitos_aplicacion.id"), nullable=False)
    esquema_contexto_id: Mapped[uuid_pkg.UUID] = mapped_column(ForeignKey("esquemas_contexto.id"), nullable=False)
    
    logica_json: Mapped[dict] = mapped_column(JSONB, nullable=False)
    template_error: Mapped[str] = mapped_column(String, nullable=False)
    vigencia: Mapped[object] = mapped_column(DATERANGE, nullable=False)
    hash_firma: Mapped[Optional[str]] = mapped_column(String, nullable=True) # Event sourcing proof
    hash_algoritmo: Mapped[str] = mapped_column(String(20), nullable=False, default="SHA-256")
    firmado_en: Mapped[Optional[datetime]] = mapped_column(DateTime(timezone=True), nullable=True)
    
    regla = relationship("ReglaIdentidad", back_populates="versiones")

    __table_args__ = (
        ExcludeConstraint(
            ('regla_uuid', '='),
            ('ambito_id', '='),
            ('vigencia', '&&'),
            name='no_solapamiento_por_ambito_vigencia'
        ),
    )

@event.listens_for(ReglaVersion, "before_update")
def prevent_update(mapper, connection, target):
    raise IntegrityError(
        statement=None,
        params=None,
        orig="ReglaVersion is immutable after insertion. To change rules, create a new version.",
    )
