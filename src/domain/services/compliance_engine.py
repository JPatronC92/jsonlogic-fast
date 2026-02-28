import logging
from string import Template
from typing import Dict, Any, List
from json_logic import jsonLogic
from jsonschema import validate, ValidationError
from src.domain.schemas.compliance import ReglaEvaluable, ResultadoEvaluacion, DetalleFallo

# A. Logging Estructurado
logger = logging.getLogger("tempus.compliance_engine")

class ComplianceEngine:
    """
    Motor determinista puro. 
    Stateless: (Contexto + Reglas) -> Resultado
    """
    
    def evaluar(self, contexto: Dict[str, Any], reglas: List[ReglaEvaluable]) -> ResultadoEvaluacion:
        errores = []
        warnings = []
        detalles_fallos = []
        reglas_ejecutadas = 0
        
        # B. Ordenar por prioridad (Las críticas primero para Fail-Fast si fuera necesario)
        reglas_ordenadas = sorted(reglas, key=lambda r: r.prioridad, reverse=True)
        
        logger.info(f"Iniciando evaluación de {len(reglas)} reglas.")
        
        for regla in reglas_ordenadas:
            try:
                # C. Validar Schema si existe antes de ejecutar lógica
                if regla.schema_validacion:
                    try:
                        validate(instance=contexto, schema=regla.schema_validacion)
                    except ValidationError as ve:
                        mensaje_error = f"Error de esquema en {regla.clave_regla}: {ve.message}"
                        logger.warning(mensaje_error)
                        detalles_fallos.append(DetalleFallo(
                            clave=regla.clave_regla,
                            severidad="ERROR",
                            mensaje=mensaje_error
                        ))
                        errores.append(mensaje_error)
                        continue # No ejecutamos lógica si el esquema falla

                # D. Ejecución de json-logic segura
                cumple = jsonLogic(regla.logica, contexto)
                
                reglas_ejecutadas += 1
                
                if not cumple:
                    # E. Interpolación Segura
                    mensaje = self._formatear_mensaje(regla.template_error, contexto)
                    
                    detalle = DetalleFallo(
                        clave=regla.clave_regla,
                        severidad=regla.severidad,
                        mensaje=mensaje
                    )
                    detalles_fallos.append(detalle)
                    
                    if regla.severidad in ['ERROR', 'BLOCKER']:
                        logger.debug(f"Regla fallida (BLOCKING): {regla.clave_regla}")
                        errores.append(mensaje)
                        # Opcional: Implementar Short-circuit aquí si es BLOCKER
                    else:
                        logger.debug(f"Regla fallida (WARNING): {regla.clave_regla}")
                        warnings.append(mensaje)
                        
            except Exception as e:
                # F. Manejo de Excepciones sin romper el flujo completo
                logger.error(f"Error crítico ejecutando regla {regla.clave_regla}: {str(e)}", exc_info=True)
                errores.append(f"Error de sistema en regla {regla.clave_regla}")
                detalles_fallos.append(DetalleFallo(clave=regla.clave_regla, severidad="ERROR", mensaje="Error de sistema"))

        es_valido = len(errores) == 0
        
        return ResultadoEvaluacion(
            es_valido=es_valido,
            score_cumplimiento=1.0 if es_valido else 0.0,
            errores=errores,
            warnings=warnings,
            reglas_ejecutadas=reglas_ejecutadas,
            detalles_fallos=detalles_fallos
        )

    def _formatear_mensaje(self, template_str: str, contexto: Dict) -> str:
        """Interpolación segura que no explota si falta una variable."""
        try:
            return Template(template_str).safe_substitute(contexto)
        except Exception:
            return template_str # Fallback seguro
