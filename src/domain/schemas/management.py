from pydantic import BaseModel, Field
from typing import Dict, Any, Optional
from datetime import date
from uuid import UUID

# --- Esquemas Contexto ---

class EsquemaContextoCreate(BaseModel):
    dominio_id: str = Field(..., description="Ej: FISCAL_RENTA")
    version: int = Field(..., description="Versión entera incremental")
    schema_json: Dict[str, Any] = Field(..., description="JSON Schema válido")

class EsquemaContextoResponse(EsquemaContextoCreate):
    id: UUID

# --- Reglas Identidad ---

class ReglaIdentidadCreate(BaseModel):
    dominio_id: str = Field(..., description="Ej: FISCAL_RENTA")
    urn_global: str = Field(..., description="Ej: urn:lex:mx:fiscal:isr:limite_deduccion")
    clave_interna: Optional[str] = Field(None, description="Legacy o interno")
    tipo_obligacion: str = Field(..., description="LIMITE, PROHIBICION, REQUISITO")
    criticidad: str = Field(..., description="INFO, WARNING, BLOCKER")

class ReglaIdentidadResponse(ReglaIdentidadCreate):
    uuid: UUID

# --- Reglas Version (Publicación) ---

class Vigencia(BaseModel):
    inicio: date
    fin: Optional[date] = None

class ReglaVersionCreate(BaseModel):
    regla_uuid: UUID
    ambito_id: UUID
    esquema_contexto_id: UUID
    logica_json: Dict[str, Any]
    template_error: str
    vigencia: Vigencia

class ReglaVersionResponse(ReglaVersionCreate):
    id: UUID
    hash_firma: str

class VerifyRuleResponse(BaseModel):
    valid: bool
    hash_match: bool
    stored_hash: str
    recalculated_hash: str
    hash_algoritmo: str
