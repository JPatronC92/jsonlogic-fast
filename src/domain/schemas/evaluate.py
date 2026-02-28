from pydantic import BaseModel, Field
from typing import Dict, Any, List
from datetime import date

class SelectorAmbito(BaseModel):
    jurisdiccion: str = Field(..., description="Ej: iso:mx")
    autoridad: str = Field(..., description="Ej: SAT")
    industria: str = Field(..., description="Ej: FINANZAS")
    dominio: str = Field(..., description="Ej: FISCAL_RENTA")

class EvaluateRequest(BaseModel):
    selector: SelectorAmbito
    fecha_operacion: date
    contexto: Dict[str, Any]

class DetalleFallo(BaseModel):
    urn_global: str
    mensaje: str
    criticidad: str

class EvaluateResponse(BaseModel):
    es_valido: bool
    reglas_ejecutadas: int
    detalles_fallos: List[DetalleFallo] = []
