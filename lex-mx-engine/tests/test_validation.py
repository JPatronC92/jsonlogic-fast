import pytest
from src.domain.services.compliance_engine import ComplianceEngine
from src.domain.schemas.compliance import ReglaEvaluable

def test_compliance_engine_validation_fail():
    engine = ComplianceEngine()
    
    # Define a rule with a schema that requires 'valor' to be a number
    regla = ReglaEvaluable(
        id_version="v1",
        clave_regla="TEST-VAL-001",
        logica={"==": [{"var": "check"}, True]},
        schema_validacion={
            "type": "object",
            "properties": {
                "valor": {"type": "number"}
            },
            "required": ["valor"]
        },
        template_error="Error",
        prioridad=100,
        severidad="ERROR"
    )
    
    # Input with 'valor' as string (should fail validation)
    contexto = {"valor": "no-soy-numero", "check": True}
    
    resultado = engine.evaluar(contexto, [regla])
    
    assert resultado.es_valido is False
    assert any("Error de esquema" in err for err in resultado.errores)
    assert resultado.reglas_ejecutadas == 0 # Logic shouldn't run if validation fails

def test_compliance_engine_validation_success():
    engine = ComplianceEngine()
    
    regla = ReglaEvaluable(
        id_version="v1",
        clave_regla="TEST-VAL-001",
        logica={"==": [{"var": "valor"}, 10]},
        schema_validacion={
            "type": "object",
            "properties": {
                "valor": {"type": "number"}
            }
        },
        template_error="Valor no es 10",
        prioridad=100,
        severidad="ERROR"
    )
    
    # Valid input
    contexto = {"valor": 10}
    resultado = engine.evaluar(contexto, [regla])
    
    assert resultado.es_valido is True
    assert resultado.reglas_ejecutadas == 1
