from src.domain.services.compliance_engine import ComplianceEngineUniversal

# Mock classes to simulate the expected models
class MockReglaIdentidad:
    def __init__(self, urn_global="urn:test", criticidad="ERROR"):
        self.urn_global = urn_global
        self.criticidad = criticidad

class MockReglaVersion:
    def __init__(self, logica_json, template_error, urn_global="urn:test", criticidad="ERROR"):
        self.logica_json = logica_json
        self.template_error = template_error
        self.regla = MockReglaIdentidad(urn_global, criticidad)

def test_compliance_engine_validation_fail():
    engine = ComplianceEngineUniversal()
    
    # Define a rule with a schema that requires 'valor' to be a number
    regla = MockReglaVersion(
        logica_json={"==": [{"var": "check"}, True]},
        template_error="Error"
    )
    esquema_validacion = {
        "type": "object",
        "properties": {
            "valor": {"type": "number"}
        },
        "required": ["valor"]
    }
    
    # Input with 'valor' as string (should fail validation)
    contexto = {"valor": "no-soy-numero", "check": True}
    
    resultado = engine.evaluar(contexto, [regla], esquema_validacion)
    
    assert resultado.es_valido is False
    assert any("SYS-SCHEMA" in fallo.urn_global for fallo in resultado.detalles_fallos)
    assert resultado.reglas_ejecutadas == 0 # Logic shouldn't run if validation fails

def test_compliance_engine_validation_success():
    engine = ComplianceEngineUniversal()
    
    regla = MockReglaVersion(
        logica_json={"==": [{"var": "valor"}, 10]},
        template_error="Valor no es 10"
    )
    esquema_validacion = {
        "type": "object",
        "properties": {
            "valor": {"type": "number"}
        }
    }
    
    # Valid input
    contexto = {"valor": 10}
    resultado = engine.evaluar(contexto, [regla], esquema_validacion)
    
    assert resultado.es_valido is True
    assert resultado.reglas_ejecutadas == 1
