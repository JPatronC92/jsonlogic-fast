# Roadmap de publicación pública (Tempus Engine)

> Objetivo: dejar el proyecto en estado **public-ready** (seguro, documentado, reproducible y mantenible) para open source y adopción de terceros.

## Criterios de salida (Definition of Done global)
- CI verde en PR y en `main`.
- Documentación de instalación y API **alineada con el código real**.
- Baseline de seguridad (secretos, dependencias, auth, disclosure).
- Suite de pruebas mínima estable y reproducible en entorno limpio.
- Versionado, changelog y release notes para primera versión pública.

---

## Fase 0 — Contención inmediata (Semana 1)

### 0.1 Higiene de módulos críticos
- [ ] Eliminar imports/definiciones duplicadas en `src/core/security.py`.
- [ ] Eliminar duplicados en `src/domain/models.py` (`get_cached_validator`, imports repetidos).
- [ ] Ejecutar `ruff`/`isort` y dejar regla para bloquear PR con duplicados obvios.

**Entrega:** código legible y sin redefiniciones accidentales en seguridad/dominio.

### 0.2 Restaurar confiabilidad de tests
- [ ] Corregir `tests/test_security.py` (estilo de import único, quitar duplicados, evitar símbolos no importados).
- [ ] Agregar casos de precedencia auth (API key > JWT) y JWT audience inválida.
- [ ] Establecer comando único de test para colaboradores (`make test` o script equivalente).

**Entrega:** suite base ejecutable en limpio y enfocada en autenticación.

### 0.3 Corregir inconsistencias de documentación
- [ ] Alinear `docs/api.md` con rutas reales (`/billing/calculate`, `/billing/simulate-batch`).
- [ ] Corregir payloads y ejemplos `curl` para reflejar request/response actuales.
- [ ] Añadir tabla “compatibilidad de endpoints” si se renombraron rutas históricas.

**Entrega:** docs sin drift respecto al router actual.

---

## Fase 1 — Calidad de ingeniería para OSS (Semana 2)

### 1.1 Pipeline de CI/CD mínimo
- [ ] Definir jobs obligatorios: lint, unit tests, build.
- [ ] Publicar matriz mínima de Python soportado.
- [ ] Configurar cache de dependencias para tiempos de CI estables.

### 1.2 Estándares de contribución
- [ ] Endurecer `CONTRIBUTING.md` con “cómo correr proyecto local + cómo testear”.
- [ ] Agregar plantilla de PR y de issues (bug, feature, docs).
- [ ] Definir convención de commits y branch naming.

### 1.3 Calidad de documentación técnica
- [ ] Revisar README “Quick Start” end-to-end con pasos validados.
- [ ] Documentar arquitectura runtime (API, DB, Rust core, SDKs).
- [ ] Agregar sección de troubleshooting (errores frecuentes de setup).

**Entrega:** experiencia de contribuidor clara y repetible.

---

## Fase 2 — Seguridad y cumplimiento (Semana 3)

### 2.1 Gestión de secretos y configuración
- [ ] Validar que `.env.example` no promueva defaults inseguros.
- [ ] Documentar rotación de `SECRET_KEY` y API keys.
- [ ] Añadir validación fuerte de settings al arranque (fail-fast).

### 2.2 Hardening de autenticación/autorización
- [ ] Revisar flujo API key hashing/consulta para evitar supuestos ambiguos.
- [ ] Añadir pruebas para credenciales expiradas/inactivas/forjadas.
- [ ] Definir política de errores (mensajes no filtren detalles sensibles).

### 2.3 Vulnerabilidades y dependencias
- [ ] Activar escaneo de dependencias en CI.
- [ ] Definir SLA de actualización para CVEs críticas/altas.
- [ ] Registrar proceso de disclosure en `SECURITY.md` (tiempos y canal).

**Entrega:** baseline de seguridad apto para exposición pública.

---

## Fase 3 — Producto public-ready (Semana 4)

### 3.1 Versionado y release management
- [ ] Definir versión inicial pública (ej. `v0.1.0` o `v1.0.0-beta`).
- [ ] Actualizar `CHANGELOG.md` con cambios verificables.
- [ ] Preparar release notes con breaking changes y guía de migración.

### 3.2 DX de SDKs y ejemplos
- [ ] Verificar ejemplos Python/Node contra API real.
- [ ] Añadir snippets completos (auth, calculate, simulate-batch, manejo de errores).
- [ ] Incluir mini “getting started” por SDK con tiempos estimados.

### 3.3 Observabilidad y operación
- [ ] Establecer logging estructurado mínimo en API.
- [ ] Definir health/readiness endpoints y su documentación.
- [ ] Checklist de despliegue (DB migrations, rollback, smoke tests).

**Entrega:** proyecto listo para adopción externa y operación básica.

---

## Backlog de alto valor (post-lanzamiento)
- [ ] Cobertura de pruebas por umbral en CI (objetivo incremental).
- [ ] Benchmarks reproducibles en pipeline.
- [ ] Contrato OpenAPI versionado y pruebas de compatibilidad hacia atrás.
- [ ] Guía de roadmap pública y governance de comunidad.

---

## Priorización sugerida (Impacto vs Esfuerzo)

### Alta prioridad / bajo esfuerzo
1. Duplicados en `security.py` y `models.py`.
2. Alineación de `docs/api.md` con rutas reales.
3. Arreglo de `tests/test_security.py`.

### Alta prioridad / esfuerzo medio
4. CI con lint + tests + build.
5. Hardening de auth y validación de settings.
6. Release process (version/changelog/release notes).

### Media prioridad / esfuerzo medio
7. Observabilidad base (logs/healthchecks).
8. Mejora de ejemplos de SDK.
9. Plantillas de contribución y governance.

---

## KPI de preparación para publicar
- Tiempo de setup local de contribuidor nuevo < 20 minutos.
- Tasa de éxito de pipeline CI > 95% en últimas 2 semanas.
- 0 vulnerabilidades críticas abiertas.
- 0 discrepancias conocidas entre docs API y rutas reales.
- Flujo de release repetible documentado (al menos 1 dry run exitoso).
