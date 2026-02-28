import pytest
from src.domain.services.compliance_engine import ComplianceEngine
from src.domain.schemas.compliance import ReglaEvaluable

# Fixtures (Datos de prueba)
@pytest.fixture
def engine():
    return ComplianceEngine()

@pytest.fixture
def regla_gasolina():
    return ReglaEvaluable(
        id_version="v1",
        clave_regla="ISR-GAS-001",
        prioridad=100,
        severidad="ERROR",
        template_error="Gasto de combustible por $monto requiere tarjeta, no $metodo_pago.",
        logica={
            "if": [
                {"and": [
                    {"==": [{"var": "tipo"}, "combustible"]},
                    {">": [{"var": "monto"}, 2000]}
                ]},
                {"in": [{"var": "metodo_pago"}, ["TARJETA", "TRANSFERENCIA"]]},
                True # Pasa si no es combustible > 2000
            ]
        }
    )

def test_gasolina_pago_invalido(engine, regla_gasolina):
    """Caso Negativo: Efectivo > 2000"""
    contexto = {"tipo": "combustible", "monto": 3000, "metodo_pago": "EFECTIVO"}
    
    resultado = engine.evaluar(contexto, [regla_gasolina])
    
    assert resultado.es_valido is False
    assert len(resultado.errores) == 1
    # Validar interpolación segura
    assert "requiere tarjeta, no EFECTIVO" in resultado.errores[0]

def test_gasolina_limite_exacto(engine, regla_gasolina):
    """Caso Borde: Exactamente 2000 (Debe pasar en efectivo)"""
    contexto = {"tipo": "combustible", "monto": 2000, "metodo_pago": "EFECTIVO"}
    
    resultado = engine.evaluar(contexto, [regla_gasolina])
    
    assert resultado.es_valido is True
    assert len(resultado.errores) == 0

def test_interpolacion_segura_variables_faltantes(engine):
    """Si el contexto no tiene la variable del mensaje, no debe explotar"""
    regla_rota = ReglaEvaluable(
        id_version="v1", clave_regla="TEST", prioridad=10, severidad="ERROR",
        template_error="Error en campo $variable_inexistente",
        logica={"==": [1, 2]} # Siempre falla
    )
    
    resultado = engine.evaluar({}, [regla_rota])
    
    assert resultado.es_valido is False
    # El mensaje debe salir raw o con placeholder vacío, pero no Exception
    assert "Error en campo $variable_inexistente" in resultado.errores[0]
