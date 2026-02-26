Esta es la **Bitácora Oficial del Proyecto** actualizada tras el sprint de hoy.
Este documento define la identidad, el alcance y el inventario técnico de lo que hemos construido.

---

# 📂 Bitácora de Proyecto: Lex MX Engine

**Fecha de Corte:** 26 de Febrero, 2026
**Estatus:** Transición a Lex API, Endpoint de Árbol Estructural y Motor de Compliance Conectado
**Repositorio:** `JPatronC92/Lex-API-Mx`

---

## 1️⃣ ¿En qué consiste el proyecto? (La Visión)

**Lex MX Engine** es Infraestructura Crítica para el cumplimiento normativo en México. Tratamos la ley como código fuente versionado, permitiendo consultas deterministas en el tiempo ("Time Travel") sin las alucinaciones típicas de los LLMs. Se ha comenzado la evolución hacia un producto API comercial ("Lex API").

---

## 2️⃣ ¿Qué tenemos construido? (El Inventario Técnico)

Hemos completado el **Pipeline de Automatización Legal**. El sistema ya no solo "recibe" datos, sino que puede "leer" el DOF, "parchear" la ley solo, y exponer un árbol estructurado para visualización.

### A. Arquitectura de Datos (El Corazón) 🫀

* **Motor Temporal:** PostgreSQL 16 con `DATERANGE` y `ExcludeConstraint`.
* **Integridad:** Validado matemáticamente el no solapamiento de vigencias mediante índices `GiST`.
* **Modelos:** Separación de Identidad Inmutable (`UnidadEstructural`/`ReglaIdentidad`) y Contenido Versionado (`VersionContenido`/`ReglaVersion`).
* **Histórico Seguro:** Implementación de *soft deletes* (`deleted_at`).

### B. Componentes de Software (El Código) 🏗️

| Componente | Archivo / Ruta | Estado | Función |
| --- | --- | --- | --- |
| **Scraper DOF** | `scrapers/dof_spider.py` | ✅ Terminado | Navega el DOF, filtra decretos y extrae texto con Playwright. |
| **Cerebro (LLM)** | `infrastructure/llm_client.py` | ✅ Mejorado | Parser experto (LiteLLM) que genera `PatchCandidate`. Limpieza robusta de Markdown JSON. |
| **Resolver Legal** | `services/resolver.py` | ✅ Mejorado | Traduce texto y rutas jerárquicas ("Titulo II > Cap I > Art 27") a UUID interno. |
| **Motor de Parcheo** | `pipeline/patcher.py` | ✅ Terminado | Ejecuta la cirugía atómica de cerrar vigencias pasadas y abrir futuras. |
| **Motor de Compliance**| `services/compliance_engine.py`| ✅ Terminado | Evaluador determinista de reglas legales aislado de IA (`json-logic-qubit`). |
| **API Compliance** | `routers/v1/compliance.py` | ✅ Conectado | Endpoint `/check` implementado. Consulta `ReglaVersion` activas y evalúa contra transacciones. |
| **API Normas** | `routers/v1/normas.py` | ✅ Nuevo | Endpoints para listar leyes y renderizar el árbol jerárquico (`/estructura`) en modo Time-Travel. |
| **API de Historia** | `routers/v1/history.py` | ✅ Mejorado | Endpoints para ver la versión activa de un artículo y su historial completo. |
| **Cliente CLI** | `scripts/lex_cli.py` | ✅ Nuevo | Cliente interactivo de terminal para visualizar el árbol de las leyes consumiendo la API. |

### C. Flujos Validados (El Pipeline End-to-End) 🧪

1. **DOF -> LLM:** El Scraper baja el texto, el LLM identifica la reforma y genera el JSON de parche.
2. **LLM -> Patcher:** El Patcher recibe el JSON, el Resolver localiza el artículo original.
3. **Patcher -> DB:** Se cierra la versión vieja (`[2026-01-01, 2026-06-01)`) y se inserta la nueva (`[2026-06-01, NULL)`).
4. **Compliance Engine:** Las reglas activas se extraen de la DB y se evalúan matemáticamente sobre un payload (Transacción ERP).
5. **Time Travel API:** Endpoint de Estructura ensambla dinámicamente un árbol de nodos (Títulos/Capítulos/Artículos) con la versión vigente en cualquier fecha solicitada.

---

## 3️⃣ Diagrama de Arquitectura Actualizado

```mermaid
graph TD
    A[DOF (Diario Oficial)] -->|Playwright Scraper| B(Texto/HTML Decreto)
    B -->|LLM Parser (LiteLLM)| C{PatchCandidate JSON}
    C -->|UnidadResolver| D[Patcher Engine]
    D -->|Transacción Atómica| E[(PostgreSQL Temporal)]
    E -->|Árbol Jerárquico| F[API REST FastAPI /estructura]
    E -->|Reglas Activas| I[API REST FastAPI /check]
    F -->|JSON Estructurado| H[ERP / Frontend Visualizador]
    I -->|JSONLogic| J[Motor Compliance]
```

---

## 4️⃣ Siguientes Pasos Inmediatos

1. **Interfaz de Validación (Backoffice):** Una UI simple (Streamlit/React) para que el humano revise y apruebe el `PatchCandidate` antes de que el Patcher lo escriba en DB (El "Human-in-the-loop").
2. **Generación de Reglas por IA:** Crear agente LLM que al aprobar un cambio en el texto (ej. subir límite de deducción), proponga un parche automático en formato `json-logic` para la `ReglaIdentidad` asociada.
