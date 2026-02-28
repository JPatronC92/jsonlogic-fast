import logging
from string import Template
from typing import Dict, Any, List
from json_logic import jsonLogic
from jsonschema import validate, ValidationError
from src.domain.schemas.evaluate import EvaluateResponse, DetalleFallo

logger = logging.getLogger("lex_mx.engine_universal")

class ComplianceEngineUniversal:
    
    def evaluar(self, contexto: Dict[str, Any], reglas: List[Any], esquema_json: Dict[str, Any]) -> EvaluateResponse:
        fallos = []
        reglas_ejecutadas = 0
        
        # 1. Gobernanza de Esquema: Validar estructura del payload
        try:
            validate(instance=contexto, schema=esquema_json)
        except ValidationError as e:
            logger.error(f"Contexto inválido para el esquema: {e.message}")
            return EvaluateResponse(
                es_valido=False, reglas_ejecutadas=0,
                detalles_fallos=[DetalleFallo(urn_global="SYS-SCHEMA", mensaje=f"Contexto malformado: {e.message}", criticidad="BLOCKER")]
            )

        # 2. Evaluación Determinista
        for regla in reglas:
            try:
                cumple = jsonLogic(regla.logica_json, contexto)
                reglas_ejecutadas += 1
                
                if not cumple:
                    mensaje = Template(regla.template_error).safe_substitute(contexto)
                    fallos.append(DetalleFallo(
                        urn_global=regla.regla.urn_global or "N/A",
                        mensaje=mensaje,
                        criticidad=regla.regla.criticidad
                    ))
            except Exception as e:
                logger.error(f"Error ejecutando json-logic: {e}")

        return EvaluateResponse(
            es_valido=len(fallos) == 0,
            reglas_ejecutadas=reglas_ejecutadas,
            detalles_fallos=fallos
        )
