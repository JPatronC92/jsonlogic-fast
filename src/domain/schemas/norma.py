from pydantic import BaseModel, Field
from datetime import date
from uuid import UUID
from typing import Optional, List

class VersionOutput(BaseModel):
    id: UUID
    texto_contenido: str
    vigencia_inicio: date = Field(..., description="Fecha de inicio de la vigencia (inclusiva)")
    vigencia_fin: Optional[date] = Field(None, description="Fecha de fin de la vigencia (exclusiva, o None para infinito)")
    nomenclatura_visible: str

class UnidadOutput(BaseModel):
    uuid: UUID
    tipo: str
    version_activa: VersionOutput # La versión que aplica en la fecha solicitada

# --- Nuevos esquemas para el API Completo ---

class NormaBase(BaseModel):
    id: UUID
    nombre_oficial: str
    nombre_corto: Optional[str]
    estado: str

class TreeNode(BaseModel):
    uuid: UUID
    tipo: str
    nomenclatura: str
    orden: float
    texto: str
    hijos: List['TreeNode'] = []

TreeNode.model_rebuild()

class HistorialUnidadResponse(BaseModel):
    uuid: UUID
    tipo: str
    versiones: List[VersionOutput]
