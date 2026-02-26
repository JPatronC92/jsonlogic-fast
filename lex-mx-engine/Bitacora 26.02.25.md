Esta es la **Bitácora Oficial del Proyecto** actualizada tras el sprint de hoy.
Este documento define la identidad, el alcance y el inventario técnico de lo que hemos construido.

---

# 📂 Bitácora de Proyecto: Lex MX Engine -> Lex API

**Fecha de Corte:** 26 de Febrero, 2026
**Estatus:** Transición a "Lex API" completada en su núcleo (Endpoints de Árbol Estructural Histórico y Compliance conectados).
**Repositorio:** `JPatronC92/Lex-API-Mx`

---

## 1️⃣ ¿En qué consiste el proyecto? (La Evolución a Lex API)

El proyecto ha evolucionado de ser únicamente un motor interno de procesamiento (**Lex MX Engine**) a convertirse en un producto B2B consumible: **Lex API**. 

**Lex API** es Infraestructura Legal Crítica ("El Stripe del Cumplimiento Normativo en México"). Tratamos la ley como código fuente versionado, permitiendo consultas deterministas en el tiempo ("Time Travel") sin las alucinaciones típicas de los LLMs. Ofrece a ERPs, CRMs y firmas legales la capacidad de consumir la ley mexicana y sus reglas de negocio a través de una API REST moderna.

---

## 2️⃣ ¿Qué tenemos construido? (El Inventario Técnico)

Hemos expuesto la potencia del motor interno mediante la nueva capa de API. El sistema lee el DOF, actualiza su base de datos temporal, y expone la ley estructurada para cualquier fecha en la historia.

### A. Arquitectura de Datos (El Corazón Temporal) 🫀

* **Motor Temporal:** PostgreSQL 16 con `DATERANGE` y `ExcludeConstraint`.
* **Integridad:** Validado matemáticamente el no solapamiento de vigencias mediante índices `GiST`.
* **Modelos:** Separación de Identidad Inmutable (`UnidadEstructural`/`ReglaIdentidad`) y Contenido Versionado (`VersionContenido`/`ReglaVersion`).

### B. La Capa "Lex API" (Los Endpoints) 🌐

Se han implementado los endpoints comerciales fundamentales:

| Endpoint | Estado | Función |
| --- | --- | --- |
| `GET /api/v1/normas` | ✅ Operativo | Catálogo de leyes y códigos disponibles en la plataforma. |
| `GET /api/v1/normas/{id}/estructura` | ✅ Operativo | **Time Travel API**. Construye el árbol jerárquico completo (Títulos > Capítulos > Artículos) ensamblando únicamente las versiones del texto vigentes en la `?fecha=` solicitada. |
| `GET /api/v1/history/articulos/{uuid}/historial` | ✅ Operativo | Auditoría Forense. Devuelve la línea de vida completa de un artículo (pasado, presente y futuro aprobado). |
| `POST /api/v1/compliance/check` | ✅ Operativo | Motor de Reglas (`json-logic`). Recibe metadata de una transacción de un ERP, busca las reglas fiscales vigentes hoy, y devuelve si es *Compliant* o las violaciones encontradas. |

### C. Componentes del Motor Interno (El Backoffice) 🏗️

| Componente | Archivo / Ruta | Función |
| --- | --- | --- |
| **Scraper DOF** | `scrapers/dof_spider.py` | Navega el DOF, filtra decretos y extrae texto. |
| **Cerebro (LLM)** | `infrastructure/llm_client.py` | Parser experto (LiteLLM) que genera `PatchCandidate`. Manejo robusto de JSON y limpieza de Markdown. |
| **Resolver Legal** | `services/resolver.py` | Traduce texto y rutas jerárquicas ("Titulo II > Cap I > Art 27") a UUID interno. |
| **Motor de Parcheo** | `pipeline/patcher.py` | Ejecuta la cirugía atómica de cerrar vigencias pasadas y abrir futuras. |
| **Cliente CLI** | `scripts/lex_cli.py` | Cliente interactivo de terminal para visualizar el árbol de las leyes consumiendo la propia API. |

---

## 3️⃣ Diagrama de Arquitectura Actualizado

```mermaid
graph TD
    %% INGESTA Y PROCESAMIENTO (El Engine)
    A[DOF (Diario Oficial)] -->|Playwright Scraper| B(Texto Decreto)
    B -->|LLM Parser (LiteLLM)| C{PatchCandidate JSON}
    C -->|UnidadResolver| D[Patcher Engine]
    D -->|Transacción Atómica| E[(PostgreSQL Temporal)]
    
    %% CONSUMO (Lex API)
    E -->|Árbol Jerárquico| F[API REST: /estructura]
    E -->|Reglas Activas| I[API REST: /check]
    E -->|Auditoría| K[API REST: /historial]
    
    %% CLIENTES FINALES
    F -->|JSON Estructurado Time-Travel| H[Frontend Visualizador / Sistemas de Consulta]
    I -->|JSONLogic| J[ERP / CRM / Software Contable]
    K -->|Auditoría Histórica| L[Firmas Legales / Auditores]
```

---

## 4️⃣ Siguientes Pasos (Roadmap de Producto)

1. **El "Human-in-the-Loop" (UI de Aprobación):** Una interfaz de Backoffice (Streamlit o React) para que el experto legal revise el `PatchCandidate` propuesto por la IA (el Diff del artículo viejo vs nuevo) y lo apruebe antes de inyectarlo en PostgreSQL.
2. **Auto-Generación de Reglas de Negocio:** Expandir el pipeline LLM para que, cuando cambie la ley, no solo modifique el texto (`VersionContenido`), sino que proponga automáticamente una actualización en la fórmula matemática (`json-logic`) de la `ReglaVersion` correspondiente.