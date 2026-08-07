# jsonlogic-ai

> **La capa de orquestación y guardrails deterministas para aplicaciones basadas en SLMs (Small Language Models) e IA Local.**

`jsonlogic-ai` es un framework de políticas de ultra-alto rendimiento construido en Rust sobre la base ultra-rápida de `jsonlogic-fast`. Permite estructurar, validar, enrutar y refinar de manera determinista las entradas y salidas de modelos de lenguaje pequeños (SLMs) u orquestaciones locales de agentes de IA con latencia en microsegundos.

---

## 🛠️ Los 3 Pilares de `jsonlogic-ai`

```
┌────────────────────────────────────────────────────────────────────────┐
│                        LIBRERÍA / FRAMEWORK                            │
├──────────────────────────┬──────────────────────┬──────────────────────┤
│ 1. Structural Guardrails │ 2. Logic-Driven RAG  │ 3. Natural Language  │
│ (Post-validación de LLM) │ (Pre-filtrado rápido)│ Rules Generator      │
└──────────────────────────┴──────────────────────┴──────────────────────┘
```

### 1. Structural Guardrails (Validación y Reflexión)
Los SLMs locales tienden a romper la estructura JSON o violar las reglas de negocio en sus respuestas. `jsonlogic-ai` permite envolver tus aserciones lógicas en una política enriquecida (`RuleEnvelope`) con metadatos específicos:
* Mensajes de error localizados y contextuales.
* Nivel de severidad (`error`, `warning`).
* Estado de reintento (`retryable: true/false`).
* Extracción automática del valor actual (`actual`) mediante JSON path.

Ofrece un bucle de reflexión automática determinista (`orchestrate_reflection` / orquestador) que envía reportes de violación estructurados de vuelta al modelo de lenguaje en caso de fallos.

### 2. Logic-Driven Context Injection & Routing
Evita la inyección masiva de contexto en la ventana de tokens de tus SLMs (lo que ralentiza la inferencia y degrada la atención). Usa un enrutador determinista (`Router`) basado en JSON-Logic para tomar decisiones de enrutamiento ultrarrápidas (<1ms) sobre metadatos estructurados de usuario o entorno.

### 3. Natural-to-Logic Compiler (Parser DSL en v0.1, Asistente LLM en v0.2/0.3)
El "Secret Sauce": un compilador híbrido compuesto por:
1. **Parser determinista de un DSL simple (Ya disponible en v0.1)**: Traduce de pseudocódigo lógico a JSON-Logic 100% offline y seguro.
2. **Asistente LLM opcional**: Convierte descripciones de reglas en lenguaje natural al DSL formal, asegurando que la salida de la IA siempre pase por validación determinista antes de ejecutarse.

---

## 📦 Características del MVP (Versión 0.1)

El MVP actual (v0.1) incluye las implementaciones canónicas nativas en **Rust** y sus correspondientes bindings de alta velocidad para **Python**:

* **`RuleEnvelope`**: Define políticas con metadatos de severidad, mensaje amigable, ruta JSON y estado re-intentable.
* **`evaluate_guardrails`**: Ejecuta aserciones JSON-Logic sobre la salida estructurada de los modelos y devuelve un reporte detallado de violaciones.
* **`Router`**: Evaluación y despacho determinista basado en contexto de usuario, entorno e intenciones.
* **`orchestrate_reflection`**: Orquestación automática del bucle de reflexión para corregir respuestas erróneas del modelo mediante reintentos con feedback estructurado, detección de bucles infinitos y soporte para abortar por fallos críticos (no corregibles).
* **`compile_dsl`**: Compila un DSL intuitivo basado en texto ("when ... then ... else ...") de vuelta a la representación nativa estructurada de JSON-Logic, de manera offline, segura y ultra rápida.

---

## 🚀 Ejemplos de Uso

### 🐍 Python (v0.1)

Aquí tienes cómo usar los guardrails, el orquestador de reintentos, el compilador DSL y el router con un callback de generación local:

```python
from jsonlogic_fast import RuleEnvelope, evaluate_guardrails, orchestrate_reflection, Router, compile_dsl

# 1. Definir nuestras políticas de negocio deterministas
rules = [
    RuleEnvelope(
        id="temperature_range",
        assert_val={"<": [{"var": "temperature"}, 40]},
        message="La temperatura reportada no puede superar los 40 grados.",
        path="$.temperature",
        severity="error",
        retryable=True
    ),
    RuleEnvelope(
        id="status_check",
        assert_val={"==": [{"var": "status"}, "success"]},
        message="El estado final de la máquina debe ser 'success'.",
        path="$.status",
        severity="error",
        retryable=True
    )
]

# 2. Bucle de Reflexión Automatizado (Orquestador)
# Definimos el generador (ejemplo mockeando un modelo de lenguaje local)
calls = 0
def local_llm_generator(messages):
    global calls
    calls += 1
    if calls == 1:
        # El modelo comete un error en el primer intento
        return '{"temperature": 45, "status": "failed"}'
    else:
        # El modelo recibe el feedback determinista y corrige su respuesta
        return '{"temperature": 22, "status": "success"}'

# Ejecutamos la orquestación con un límite de 3 reintentos
result = orchestrate_reflection(
    rules=rules,
    generate_fn=local_llm_generator,
    initial_messages=[{"role": "user", "content": "Analiza la telemetría del motor."}],
    max_retries=3
)

if result.success:
    print("✅ ¡Validación exitosa!")
    print("Resultado corregido por el modelo:", result.output)
else:
    print("❌ Fallo en la orquestación:", result.error_message)

# 3. Router Determinista
router = Router({
    "if": [
        {"var": "request.contains_pii"},
        "anonymize_pipeline",
        {
            "if": [
                {"var": "environment.local_model_available"},
                "local_llm",
                "cloud_llm"
            ]
        }
    ]
})

decision = router.route({
    "request": {"contains_pii": False},
    "environment": {"local_model_available": True}
})
print("Decisión del Router:", decision) # 'local_llm'

# 4. Compilador DSL de Texto a JSON-Logic
dsl_rule = """
    rule approve_credit:
      when score > 700
      then "approve"
      else "review"
"""
compiled_logic = compile_dsl(dsl_rule)
print("JSON-Logic Compilado:", compiled_logic)
# Salida: {'if': [{'>': [{'var': 'score'}, 700.0]}, 'approve', 'review']}
```

### 🦀 Rust (v0.1)

```rust
use jsonlogic_fast::policy::{RuleEnvelope, Severity};
use jsonlogic_fast::guardrails::evaluate_guardrails;
use jsonlogic_fast::router::Router;
use serde_json::json;

fn main() {
    // Definir la regla de política
    let rule = RuleEnvelope {
        id: "risk_check".to_string(),
        assert: json!({"<": [{"var": "score"}, 10]}),
        message: "El nivel de riesgo detectado es crítico.".to_string(),
        path: Some("$.score".to_string()),
        severity: Severity::Error,
        retryable: true,
    };

    // Validar salida del LLM
    let output = json!({ "score": 12 });
    let report = evaluate_guardrails(&[rule], &output).unwrap();

    if !report.valid {
        println!("Violación de política detectada: {}", report.violations[0].message);
    }
}
```

---

## 🗺️ Roadmap de Versiones

### 0.1 (MVP - Actual)
* Modelos y envoltorios de políticas en Rust y Python.
* Reportes estructurados de violaciones y renderizado de prompts de feedback.
* Enrutador determinista ultra rápido.
* Orquestador nativo del bucle de reintentos/reflexión.

### 0.2 (Siguiente versión)
* Soporte para WASM / TypeScript.
* Trazas detalladas de ejecución.

### 0.3
* Compilador asistido por IA (Natural-to-Logic).
* Prompts de sistema few-shot listos para usar en local.
* Generación automática de vectores de prueba a partir de lenguaje natural.
