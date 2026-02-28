from pydantic import BaseModel, Field
from typing import Any, Dict, List, Literal, Optional
from datetime import date

class ReglaEvaluable(BaseModel):
    """DTO puro para alimentar el motor."""
    id_version: str
    clave_regla: str
    logica: Dict[str, Any]
    schema_validacion: Optional[Dict[str, Any]] = None # JSON Schema para validar contexto
    template_error: str
    prioridad: int
    severidad: Literal['INFO', 'WARNING', 'ERROR', 'BLOCKER']
    
class DetalleFallo(BaseModel):
    clave: str
    severidad: str
    mensaje: str

class UniversalEvaluateRequest(BaseModel):
    transaccion: Dict[str, Any]
    fecha_operacion: date
    
class ResultadoEvaluacion(BaseModel):
    es_valido: bool
    score_cumplimiento: float # 0.0 a 1.0
    errores: List[str]
    warnings: List[str]
    reglas_ejecutadas: int
    detalles_fallos: List[DetalleFallo] = Field(default_factory=list)
