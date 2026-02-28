from pydantic import BaseModel, Field, field_validator
from datetime import date
from typing import Optional, Literal, Dict, Any
from uuid import UUID

class VigenciaPatch(BaseModel):
    inicio: date
    fin: Optional[date] = None

class PatchCandidate(BaseModel):
    """
    Payload que genera el LLM y aprueba el Humano.
    """
    decreto_fuente_url: str
    fecha_publicacion_dof: date

    # Identificación
    norma_objetivo: str = Field(..., description="Nombre de la ley, ej: Ley del ISR")
    # Nota: El UUID se resuelve en el backend, el LLM sugiere el path textual
    unidad_path_sugerido: str = Field(..., description="Path legible: Titulo II > Cap I > Art 27")

    accion: Literal['NUEVA_VERSION', 'DEROGACION', 'ALTA_NUEVA', 'FE_ERRATAS']

    texto_nuevo: Optional[str] = None
    nomenclatura_nueva: Optional[str] = None # Por si cambia el número del artículo

    vigencia_propuesta: VigenciaPatch

    condiciones_json: Optional[Dict[str, Any]] = None # JSON Logic opcional

    # Métricas de incertidumbre (Auditoría)
    confidence_score: float = Field(..., ge=0, le=1, description="Nivel de certeza del modelo (0.0 - 1.0)")
    flag_ambiguedad: bool = Field(default=False, description="True si el decreto es confuso o contradictorio")
    raw_reasoning: str = Field(..., description="Cadena de pensamiento (CoT) del modelo antes de generar el JSON")

    @field_validator('texto_nuevo')
    def texto_obligatorio_si_no_es_derogacion(cls, v, info):
        accion = info.data.get('accion')
        if accion != 'DEROGACION' and not v:
            raise ValueError('El texto nuevo es obligatorio si no es una derogación')
        return v
