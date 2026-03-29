# Auditoría rápida del código (2026-03-28)

Este documento propone 4 tareas accionables a partir de una revisión puntual del código base.

## 1) Tarea: corregir error tipográfico
- **Tipo:** Error tipográfico.
- **Problema detectado:** En `src/core/security.py` aparece el comentario en inglés “In a prod environment...”. Para la documentación/comentarios del módulo (predominantemente en español), “prod” es una abreviatura coloquial y no consistente.
- **Propuesta:** Reescribir el comentario a una forma clara y consistente, por ejemplo “En un entorno de producción...”.
- **Impacto:** Mejora legibilidad y consistencia editorial del código.
- **Criterio de aceptación:** No quedan abreviaturas coloquiales en comentarios críticos de seguridad dentro del módulo.

## 2) Tarea: corregir una falla funcional en autenticación
- **Tipo:** Falla funcional / deuda técnica crítica.
- **Problema detectado:** `src/core/security.py` contiene múltiples imports y definiciones duplicadas (`CryptContext`, `pwd_context`, `verify_password`), lo que incrementa riesgo de inconsistencias y errores de mantenimiento.
- **Propuesta:** Consolidar imports y dejar una única definición por símbolo (`pwd_context`, `verify_password`, etc.), manteniendo comportamiento explícito y estable.
- **Impacto:** Reduce riesgo de regresiones en autenticación y simplifica auditoría de seguridad.
- **Criterio de aceptación:** El archivo compila sin duplicados lógicos y pasa lint para imports/definiciones repetidas.

## 3) Tarea: corregir discrepancia entre documentación y API real
- **Tipo:** Discrepancia de documentación.
- **Problema detectado:** `docs/api.md` documenta endpoints `/billing/evaluate` y `/billing/batch`, pero el router real expone `/billing/calculate` y `/billing/simulate-batch`.
- **Propuesta:** Actualizar `docs/api.md` para reflejar rutas, payloads y respuestas vigentes del código.
- **Impacto:** Evita errores de integración en SDKs/clientes y reduce tickets de soporte.
- **Criterio de aceptación:** Los ejemplos de `curl` y contratos de respuesta en docs coinciden con el comportamiento actual del router.

## 4) Tarea: mejorar una prueba
- **Tipo:** Mejora de testing.
- **Problema detectado:** `tests/test_security.py` depende de un símbolo `security` no importado (aunque usa `security.create_access_token`), además de imports duplicados.
- **Propuesta:** Refactorizar el test para usar un único estilo de import (módulo completo o funciones), eliminar duplicados y agregar caso explícito de precedencia de credenciales (API key vs JWT) en `get_current_tenant`.
- **Impacto:** Suite más confiable y cobertura más precisa de la lógica de autenticación.
- **Criterio de aceptación:** El archivo de pruebas es consistente, legible y cubre escenarios de éxito/fracaso + precedencia de autenticación.
