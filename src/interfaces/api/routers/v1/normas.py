from fastapi import APIRouter, HTTPException, Query
from typing import List
from datetime import date
from uuid import UUID
from src.interfaces.api.dependencies import SessionDep
from src.infrastructure.repository import TemporalRepository
from src.domain.schemas.norma import NormaBase, TreeNode

router = APIRouter()

@router.get("", response_model=List[NormaBase])
async def list_normas(session: SessionDep):
    """
    Obtiene el listado de todas las leyes y códigos disponibles.
    """
    repo = TemporalRepository(session)
    normas = await repo.get_all_normas()
    return [
        NormaBase(
            id=n.id,
            nombre_oficial=n.nombre_oficial,
            nombre_corto=n.nombre_corto,
            estado=n.estado
        )
        for n in normas
    ]

@router.get("/{id}/estructura", response_model=List[TreeNode])
async def get_estructura_norma(
    id: UUID,
    fecha: date = Query(default_factory=date.today, description="Fecha de consulta (YYYY-MM-DD)"),
    session: SessionDep = None
):
    """
    Obtiene el árbol jerárquico completo de una ley vigente a una fecha específica.
    Ideal para renderizar el índice de la ley en el frontend.
    """
    repo = TemporalRepository(session)
    items = await repo.get_estructura_norma_by_date(id, fecha)
    
    if not items:
        raise HTTPException(
            status_code=404, 
            detail=f"No se encontró estructura para la norma {id} en la fecha {fecha}"
        )

    # 1. Mapeo a nodos
    nodes_map = {}
    for unidad, version in items:
        node = TreeNode(
            uuid=unidad.uuid,
            tipo=unidad.tipo_unidad,
            nomenclatura=version.nomenclatura_visible,
            orden=unidad.orden_indice,
            texto=version.texto_contenido,
            hijos=[]
        )
        nodes_map[unidad.uuid] = {
            "node": node,
            "padre_id": unidad.padre_id
        }

    # 2. Ensamblar Árbol
    roots = []
    for uuid, data in nodes_map.items():
        padre_id = data["padre_id"]
        node = data["node"]
        
        if padre_id is None:
            roots.append(node)
        elif padre_id in nodes_map:
            nodes_map[padre_id]["node"].hijos.append(node)

    # Ordenar hijos de cada nodo y raíces
    def sort_tree(nodes: List[TreeNode]):
        nodes.sort(key=lambda x: x.orden)
        for n in nodes:
            sort_tree(n.hijos)

    sort_tree(roots)
    return roots
