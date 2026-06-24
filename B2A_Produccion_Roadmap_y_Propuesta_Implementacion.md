# Propuesta de Implementación Integral y Roadmap hacia Producción
## Proyecto B2A API - jsonlogic-fast

**Fecha:** 22 de junio de 2026  
**Rama principal de desarrollo:** `feature-b2a-api-5788506719083704110` (todo se mergea aquí, no a `main`)  
**Audiencia:** Equipo de desarrollo + stakeholders técnicos y de negocio

---

## 1. Resumen Ejecutivo

Este documento presenta una **propuesta integral** para llevar el sistema de **Logic-as-a-Service (LaaS)** para la economía de agentes de IA desde su estado actual hasta un primer despliegue en producción real sobre Base mainnet.

El proyecto no busca ser un SaaS tradicional. Su propósito es vender **decisiones lógicas deterministas y ultra-rápidas** que agentes autónomos pueden consumir y pagar segundo a segundo desde su propio staking criptográfico.

El trabajo realizado hasta ahora (StorageBackend + DynamoDB, integración Alloy, Terraform, pipeline CI/CD, migración a `U256` y el modelo de staking + slasher) es una base técnica sólida para el modelo B2A.

Sin embargo, varios de los gaps que identificamos tienen un peso diferente cuando se mira el proyecto bajo la lente de **infraestructura lógica para agentes económicos autónomos**.

**Objetivo principal:** Alcanzar un despliegue en producción seguro, observable y mantenible en un horizonte de **6-8 semanas** (asumiendo dedicación media-alta).

---

## 2. Estado Actual del Proyecto

### Componentes implementados
- API serverless (Axum + Lambda arm64)
- Autenticación SIWE básica
- Motor de pricing por profundidad de regla
- Almacenamiento dual (Memory + DynamoDB)
- Sincronización de depósitos (sync_deposits) usando Alloy
- Motor de slashing (slasher)
- Infraestructura como código (Terraform)
- Pipeline de CI/CD básico (GitHub Actions + cargo-lambda)

### Componentes faltantes o incompletos
- El binario `sync_deposits` **no está incluido** en el pipeline ni en Terraform.
- Validación SIWE es débil (`VerificationOpts::default()`).
- Deducción de saldo no es atómica (posibles race conditions).
- El contrato `slash` no tiene control de acceso.
- Ausencia de monitoreo, alerting y observabilidad.
- Gestión de secretos deficiente (PRIVATE_KEY en variables de entorno).
- No hay protección contra abuso ni rate limiting visible.

---

## 3. Modelo de Monetización Actual y Propuesto (Peaje Criptográfico + Royalty)

### Modelo Actual (Peaje por Computación)
Actualmente el sistema cobra exclusivamente por el esfuerzo computacional:
- `base_compute_cost` calculado por `rule_depth` + `batch_size`.
- Descuento directo del saldo en staking vía `deduct_balance` antes de entregar el resultado.
- Esto ya es un modelo **trustless** para el cobro de cómputo.

### Propuesta de Royalty Automático (0.5% sobre beneficio)

Se propone evolucionar hacia un modelo donde el backend actúe como **peaje inteligente**:

1. El agente envía datos brutos (precios, capital).
2. La regla JSONLogic devuelve no solo viabilidad, sino también `beneficio_calculado`.
3. El backend extrae el beneficio, calcula 0.5% como royalty y lo suma al costo de cómputo.
4. Se descuenta el **total** del staking antes de entregar el resultado.
5. Solo si hay saldo suficiente se devuelve la decisión.

Esto garantiza cobro incluso cuando el agente gana dinero.

#### Ventajas
- Cobro garantizado (imposible evadir declarando ganancia 0).
- Modelo freemium dinámico: barato buscar, rentable cuando hay ganancia real.
- Alineado con "vender el cerebro lógico" y no solo CPU.

#### Riesgos y Consideraciones Críticas
- **Acoplamiento fuerte**: El motor lógico deja de ser puro y pasa a calcular P&L del agente.
- **Problema de datos**: El agente controla los precios que envía. Puede manipular el `beneficio_calculado`.
- **Complejidad de reglas**: Los agentes deben escribir reglas que devuelvan estructuras de beneficio precisas.
- **Precisión financiera**: Requiere manejo estricto con U256 (ya estamos en ese camino).
- **Alcance del motor**: jsonlogic-fast pasaría de ser un evaluador genérico a un motor de decisiones financieras.

**Mi recomendación actual**: 
No implementar el royalty dentro de `/v1/evaluate` en el corto plazo. 

Es más seguro y mantenible mantener el motor enfocado en **decisión + costo de cómputo**. 

Para royalty sobre beneficio, las opciones más sólidas son:
- On-chain settlement (después de que el agente ejecute la operación real y emita un evento).
- Un endpoint dedicado `/v1/settle_profit` con pruebas criptográficas o atestaciones.
- Soporte opcional de "profit hints" en las reglas, pero sin confiar ciegamente en ellos para el cobro.

El peaje por cómputo + staking + slashing ya es un modelo muy poderoso y trustless.

---

## 5. Riesgos Identificados Pendientes

### Críticos (bloqueantes para producción)
1. **Protección contra replay attacks en SIWE** depende en gran medida del TTL de DynamoDB (no inmediato).
2. **`sync_deposits` no está desplegado** → los depósitos on-chain no se reflejan en el sistema.
3. **Validación SIWE insuficiente** (sin verificación de domain, uri, chainId ni ventana temporal).
4. **Contrato sin control de acceso** en la función `slash`.

### Altos
- Deducción no atómica → posible double-spend en concurrencia alta.
- Privilegios IAM mejorables (aún se requiere `GetItem` en el API).
- Default balance de 10 unidades para cualquier wallet.
- Falta de logging estructurado y métricas de negocio.

### Medios
- Acciones de GitHub desactualizadas.
- Ausencia de tests de integración on-chain y de slasher.
- Gestión de claves privadas insegura.

---

## 4. Propuesta de Implementación Integral

### 4.1 Seguridad y Lógica de Negocio

| ID | Tarea | Prioridad | Esfuerzo estimado | Notas |
|----|-------|-----------|-------------------|-------|
| S1 | Fortalecer `verify_siwe` con `VerificationOpts` personalizado (domain, uri, chainId, issuedAt) | Crítica | 2-3 días | Mover lógica a un solo lugar |
| S2 | Eliminar doble parseo de SIWE message | Alta | 1 día | Unificar en la función de verificación |
| S3 | Implementar deducción atómica con `ConditionExpression` en DynamoDB | Alta | 3-4 días | Permitirá quitar `GetItem` del IAM |
| S4 | Agregar limpieza de nonces en MemoryStorage (o TTL lógico) | Media | 1 día | Prevenir fuga de memoria |
| S5 | Eliminar o condicionar el balance por defecto de 10 unidades | Media | 1 día | Solo dar crédito después de depósito verificado |
| S6 | Agregar rate limiting básico (por wallet o IP) | Alta | 2-3 días | Usar DynamoDB o middleware |

### 4.2 Infraestructura y Despliegue

| ID | Tarea | Prioridad | Esfuerzo | Notas |
|----|-------|-----------|----------|-------|
| I1 | Agregar build y despliegue de `sync_deposits` | Crítica | 2 días | Nuevo binario Lambda o servicio persistente |
| I2 | Crear Terraform para Sync Worker (EventBridge + Lambda o ECS/Fargate) | Crítica | 3-4 días | Debe soportar reintentos y last processed block |
| I3 | Mejorar gestión de secretos (AWS Secrets Manager + KMS) | Alta | 2 días | Quitar PRIVATE_KEY de variables |
| I4 | Actualizar GitHub Actions (checkout@v4, versiones modernas) | Media | 1 día | + agregar build de sync |
| I5 | Separar entornos (dev / staging / prod) en Terraform | Alta | 2-3 días | Usar workspaces o estructura por carpeta |
| I6 | Agregar CloudWatch Alarms + SNS para alertas críticas | Media | 2 días | Errores de slasher, fallos de sync, etc. |

### 4.3 Contrato Inteligente

| ID | Tarea | Prioridad | Esfuerzo | Notas |
|----|-------|-----------|----------|-------|
| C1 | Agregar control de acceso a `slash` (Ownable o lista de slashers autorizados) | Crítica | 1 día (Solidity) + 2 días (tests) | |
| C2 | Desplegar contrato en Base Mainnet | Alta | 1 día | + verificar |
| C3 | (Recomendado) Auditoría ligera del contrato | Alta | Externo | Presupuesto separado |

### 4.4 Observabilidad, Operaciones y Hardening

- Logging estructurado (JSON) + correlación de requests.
- Métricas clave: evaluaciones por minuto, costo promedio, fallos de nonce, balance por usuario (agregado).
- Implementar health checks y readiness.
- Documentación de runbooks (qué hacer si falla el slasher).
- Estrategia de versionado de la API (`/v1` actual → plan para v2).

---

## 5. Roadmap hacia Primer Despliegue en Producción

### Fase 0 - Actual (ya hecho)
- Arquitectura base + U256 + Terraform inicial.

### Fase 1 - Seguridad Crítica y Confianza en el Peaje (Semanas 1-2)
- S1, S2, S3, C1
- Fortalecimiento de deducción atómica (base para cualquier modelo de cobro)
- **Entregable:** Peaje por cómputo 100% confiable y trustless

### Fase 2 - Infraestructura Completa (Semanas 2-3)
- I1, I2, I3, I4
- Primer despliegue en entorno **staging**
- **Entregable:** Sync worker funcionando + secretos gestionados

### Fase 3 - Hardening, Pruebas y Evaluación de Royalty (Semanas 3-4)
- S4, S5, S6, I5, I6
- Pruebas de carga y slashing
- **Estudio y prototipo** del modelo de royalty automático (ver sección 3)
- Decisión estratégica: ¿implementar royalty dentro del motor lógico o vía mecanismo on-chain/separado?
- **Entregable:** Staging estable + decisión documentada sobre royalty

### Fase 4 - Pre-Producción (Semana 5)
- Despliegue del contrato en Base Mainnet
- Configuración de secrets de producción
- Revisión final de IAM y modelo de cobro
- **Entregable:** Checklist de go-live completado

### Fase 5 - Go-Live Producción (Semana 6-8)
- Despliegue en la rama feature-b2a + Terraform apply
- Monitoreo intensivo las primeras 72-96 horas
- Lanzamiento inicial con modelo de **solo cómputo** (recomendado)
- **Entregable:** Sistema en producción operando bajo el modelo B2A para agentes autónomos

---

## 6. Requisitos Previos y Configuraciones Necesarias (antes de Producción)

### 6.1 AWS
- Cuenta AWS con billing activado
- Usuario IAM con permisos limitados para CI/CD (no root)
- Secrets configurados:
  - `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY`
  - `PRIVATE_KEY` (preferiblemente rotado regularmente)
- Opcional pero recomendado: AWS KMS para firmar transacciones

### 6.2 Blockchain
- Despliegue del contrato `B2AStaking` en **Base Mainnet**
- Dirección del contrato registrada en variables de Terraform
- Fondos en la wallet del slasher (gas + margen)
- Recomendación: Wallet separada para el slasher (nunca la misma que el deployer)

### 6.3 Dominio y Acceso
- (Recomendado) Dominio propio (ej: `api.b2a.jsonlogic-fast.com`)
- Certificado (ACM) + Route53 o CloudFront si se desea
- Documentación pública de la API actualizada

### 6.4 Monitoreo y Alertas
- Suscripción SNS + email / Slack / PagerDuty
- Definición de SLOs iniciales (ej: 99.5% uptime, p95 < 800ms)

### 6.5 Otros
- Repositorio de issues / backlog organizado
- Proceso de rotación de claves privadas
- Política de respuesta a incidentes (mínima)

---

## 7. Proyección Económica

### 7.1 Costos de Desarrollo (estimados)

| Fase | Esfuerzo estimado | Costo aproximado (a $80/h) | Notas |
|------|-------------------|-----------------------------|-------|
| Fase 1 (Seguridad) | 10-12 días persona | $6,400 - $7,680 | Incluye tests y refactor |
| Fase 2 (Infra) | 8-10 días persona | $5,120 - $6,400 | Sync + secretos |
| Fase 3 (Hardening + pruebas) | 7-9 días persona | $4,480 - $5,760 | Carga + slashing tests |
| Fase 4-5 | 4-5 días persona | $2,560 - $3,200 | Go live + soporte inicial |
| **Subtotal Desarrollo** | **29-36 días persona** | **$18,500 - $23,000** | |

**Recomendación:** Agregar buffer del 20-25% por imprevistos.

### 7.2 Costos de Infraestructura AWS (estimados mensuales)

**Escenario conservador inicial** (hasta 100.000 evaluaciones/mes):

| Servicio | Estimación mensual | Notas |
|----------|--------------------|-------|
| Lambda (API + Slasher + Sync) | $8 - $18 | Arm64 es muy barato |
| API Gateway HTTP | $1 - $4 | |
| DynamoDB (PAY_PER_REQUEST) | $3 - $12 | Con TTL ayuda mucho |
| EventBridge + CloudWatch | $2 - $6 | |
| Secrets + KMS (básico) | $1 - $3 | |
| **Total AWS estimado** | **$15 - $45 / mes** | Muy escalable |

**Escenario medio** (1 millón de evaluaciones/mes): ~$80-150/mes.

### 7.3 Costos Blockchain (Base)

- Gas del slasher: muy bajo en Base.
- Estimación: <$5-15 por mes al inicio (depende de frecuencia de slashing).
- Costo de despliegue del contrato en mainnet: ~$5-15 una sola vez.

### 7.4 Otros Costos One-Time

- Auditoría ligera del contrato: $2,000 - $5,000 (recomendado)
- Dominio + certificado: $15-60 / año
- Herramientas de monitoreo adicionales (si se sale de CloudWatch): variable

### 7.5 Resumen Económico (Modelo Actual - Solo Cómputo)

| Concepto | Estimación |
|----------|------------|
| Desarrollo hasta primer go-live | **$20,000 - $28,000** |
| Costo AWS mensual (inicio) | **$20 - $50** |
| Costo AWS mensual (medio) | **$80 - $150** |
| Gas + blockchain mensual | **<$20** |
| **Costo mensual total estimado (fase inicial)** | **<$100** |

### 7.6 Proyección con Royalty Automático (Escenario de Alto Impacto)

Si se implementara exitosamente el cobro de 0.5% sobre beneficio:

- En escenarios donde los agentes generan volumen significativo de arbitraje rentable, los ingresos por royalty pueden ser **10x–50x** superiores al cobro puro por cómputo.
- Ejemplo: Si agentes mueven $2M de volumen mensual con 0.8% de spread promedio → beneficio total ≈ $16,000 → 0.5% royalty = **$80/mes por agente activo**. Con 50 agentes activos → **$4,000+/mes** solo en royalty.
- Esto cambia radicalmente la unit economics del proyecto.

**Conclusión económica**: El peaje por cómputo es suficiente para empezar y ya es viable. El royalty es el multiplicador que puede convertir el proyecto en un negocio de alto margen. Sin embargo, su implementación técnica tiene riesgos altos de acoplamiento y manipulación de datos.

**Recomendación**: Priorizar primero un modelo robusto de cobro por cómputo + slashing. Evaluar royalty como Fase 2 o mediante un mecanismo on-chain separado.

---

## 8. Criterios de Éxito para Despliegue en Producción

- 100% de los tests pasando (unit + integración).
- Slasher ejecutándose exitosamente en staging con depósitos reales.
- Sync worker procesando eventos sin pérdida (al menos 48h de prueba).
- Política IAM siguiendo principio de mínimo privilegio.
- Validación SIWE completa + pruebas de replay.
- Al menos un runbook documentado.
- Monitoreo y alertas básicas activas.
- Contrato desplegado en mainnet con control de acceso.
- Documentación actualizada reflejando el modelo B2A (cobro automático desde staking).
- Decisión tomada y documentada respecto al modelo de royalty.

---

## 9. Riesgos y Mitigaciones

- **Riesgo:** Complejidad de hacer deducción atómica correctamente.
  - **Mitigación:** Implementar primero en Dynamo y mantener lógica simple en Memory.

- **Riesgo:** Sync worker pierde eventos (WebSocket inestable).
  - **Mitigación:** Implementar persistencia de último bloque procesado + reintentos + backfill histórico al inicio.

- **Riesgo:** Costos de gas o slashing imprevistos.
  - **Mitigación:** Monitoreo agresivo de balances del slasher + alertas.

- **Riesgo:** Ataque de abuso (muchas evaluaciones baratas).
  - **Mitigación:** Rate limiting + pricing agresivo + posible whitelist inicial.

---

## 10. Próximos Pasos Recomendados

1. Revisar y aprobar este documento.
2. Priorizar **Fase 1** (seguridad SIWE + deducción atómica).
3. Decidir estrategia para el Sync Worker (Lambda + EventBridge vs servicio siempre activo).
4. Programar despliegue del contrato en mainnet + revisión de ownership.
5. Definir presupuesto y recursos para las próximas 6-8 semanas.

---

**Documento actualizado** (22 junio 2026)

## 11. Fase 4 - Go-Live Checklist (Pre-Producción / Mainnet Readiness)

This section was added during Fase 4 implementation on feature-b2a-fase4-operacion.

### Pre-Deployment Gates
- [ ] cargo test (api/) + unit tests for retry/backoff pass with no regressions.
- [ ] All Fase 3 hardening (pure module, thin wrappers, real asserts, clean tree) merged and verified in source-of-truth.
- [ ] Terraform plan succeeds for environment=prod (no apply without review).
- [ ] Final IAM least-privilege review for Lambda roles (sync, slasher, api) - no wildcards on resources.
- [ ] Monitoring & alarms configured and tested (CloudWatch for sync failures, slasher errors, rate limits).
- [ ] Retry/backoff active in sync_deposits and slasher for get_block_number, get_logs, balances.call, slash.send.

### Contract & On-Chain
- [ ] B2AStaking contract deployed on **Base Mainnet** (use b2a_smart_contract.sol).
- [ ] CONTRACT_ADDRESS updated in prod secrets / TF vars for mainnet.
- [ ] Ownership / access control verified on mainnet contract (no test keys).
- [ ] Sample mainnet RPC verified: https://mainnet.base.org (or Alchemy/Infura equiv).

### Secrets & Config (Prod)
- [ ] AWS Secrets created/updated:
  - b2a/slasher-private-key-prod
  - b2a/api-private-key-prod (if separate)
  - Any RPC keys if using authenticated provider.
- [ ] Secrets never in git, TF state, or logs. Loaded only at runtime via Secrets Manager.
- [ ] Prod env vars: RPC_URL=https://mainnet.base.org , ENVIRONMENT=prod , USE_DYNAMODB=true , SYNC_MAX_BLOCKS tuned.

### Post-Deploy / Ops
- [ ] First sync worker run on prod observes real deposits (test with small on-chain tx).
- [ ] Slasher round executes without error on prod balances; watch tx confirmation.
- [ ] CloudWatch logs + alarms fire correctly on injected failures (retry path exercised).
- [ ] Go-live decision documented (sign-off on checklist above + metrics).

### Test Gates (before any mainnet funds)
- All unit + integration tests (Memory + Dynamo when possible) green.
- Manual smoke via python clients on test endpoint if sandbox available.
- No direct changes to main; all via PR to source-of-truth branch.

Update this checklist as items complete. Reference: Fase 4 items (resilience, mainnet prep, checklist) in feature-b2a-fase4-operacion.
