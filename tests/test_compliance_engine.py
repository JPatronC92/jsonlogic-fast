import pytest
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


# Fixtures (Datos de prueba)
@pytest.fixture
def engine():
    return ComplianceEngineUniversal()

@pytest.fixture
def esquema_basico():
    return {
        "type": "object",
        "properties": {
            "tipo": {"type": "string"},
            "monto": {"type": "number"},
            "metodo_pago": {"type": "string"}
        }
    }

@pytest.fixture
def regla_gasolina():
    return MockReglaVersion(
        template_error="Gasto de combustible por $monto requiere tarjeta, no $metodo_pago.",
        logica_json={
            "if": [
                {"and": [
                    {"==": [{"var": "tipo"}, "combustible"]},
                    {">": [{"var": "monto"}, 2000]}
                ]},
                {"in": [{"var": "metodo_pago"}, ["TARJETA", "TRANSFERENCIA"]]},
                True # Pasa si no es combustible > 2000
            ]
        },
        urn_global="urn:lex:mx:fiscal:isr:deduccion_combustible"
    )

def test_gasolina_pago_invalido(engine, regla_gasolina, esquema_basico):
    """Caso Negativo: Efectivo > 2000"""
    contexto = {"tipo": "combustible", "monto": 3000, "metodo_pago": "EFECTIVO"}
    
    resultado = engine.evaluar(contexto, [regla_gasolina], esquema_basico)
    
    assert resultado.es_valido is False
    assert len(resultado.detalles_fallos) == 1
    # Validar interpolación segura
    assert "requiere tarjeta, no EFECTIVO" in resultado.detalles_fallos[0].mensaje

def test_gasolina_limite_exacto(engine, regla_gasolina, esquema_basico):
    """Caso Borde: Exactamente 2000 (Debe pasar en efectivo)"""
    contexto = {"tipo": "combustible", "monto": 2000, "metodo_pago": "EFECTIVO"}
    
    resultado = engine.evaluar(contexto, [regla_gasolina], esquema_basico)
    
    assert resultado.es_valido is True
    assert len(resultado.detalles_fallos) == 0

def test_interpolacion_segura_variables_faltantes(engine, esquema_basico):
    """Si el contexto no tiene la variable del mensaje, no debe explotar"""
    regla_rota = MockReglaVersion(
        template_error="Error en campo $variable_inexistente",
        logica_json={"==": [1, 2]} # Siempre falla
    )
    
    resultado = engine.evaluar({}, [regla_rota], esquema_basico)
    
    assert resultado.es_valido is False
    # El mensaje debe salir raw o con placeholder vacío, pero no Exception
    assert "Error en campo $variable_inexistente" in resultado.detalles_fallos[0].mensaje
