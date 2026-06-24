# Propuesta de ImplementaciÔö£Ôöén Integral y Roadmap hacia ProducciÔö£Ôöén
## Proyecto B2A API - jsonlogic-fast

**Fecha:** 22 de junio de 2026 (actualizado con auditorÔö£┬ía 24 de junio 2026)  
**Rama principal de desarrollo:** `feature-b2a-api-5788506719083704110` (todo se mergea aquÔö£┬í, no a `main`)  
**Audiencia:** Equipo de desarrollo + stakeholders tÔö£┬«cnicos y de negocio

---

## ├ö┬ú├á Estado de Fases Implementadas (AuditorÔö£┬ía 24 junio 2026)

**Rama actual:** `feature-b2a-api-5788506719083704110`

**Progreso confirmado por auditorÔö£┬ía de cÔö£Ôöédigo, tests, Terraform e historia git:**

| Fase | Nombre | Estado | Evidencia principal |
|------|--------|--------|---------------------|
| 0 | Arquitectura base | ├ö┬ú├á | StorageBackend, U256, pricing, API core |
| 1 | Seguridad CrÔö£┬ítica y Confianza en el Peaje | ├ö┬ú├á | S1 (verify_siwe domain+uri + post-check + env), S2 (sin doble parse), S3 (deduct atÔö£Ôöémico ConditionExpression + retry), C1 (onlyOwner en contrato) |
| 2 | Infraestructura Completa | ├ö┬ú├á | I1-I4: sync_deposits + slasher en pipeline+TF, EventBridge, Secrets Manager, GitHub Actions modernos |
| 3 | Hardening, Pruebas y EvaluaciÔö£Ôöén | ├ö┬ú├á | S4-S6 (nonce cleanup, default=0, rate limit), I5 (environments), I6 (CloudWatch alarms + SNS) |
| 4 | Pre-ProducciÔö£Ôöén / Mainnet Readiness | ├ö┬ú├á | retry_with_backoff + tests, last_sync_block persist, structured logs, stress client, go-live checklist, mainnet comments, Fase 4 merges |

**Tests verificados (cargo test):** api_tests (6/6 ok incluyendo replay + rate limit + zero balance), core (48+ tests), hardening adapters.

**Binarios y despliegue:** api, slasher y **sync_deposits** construidos en CI/deploy.yml.

**PrÔö£Ôöéxima:** Fase 5 (Go-Live real en prod + mainnet contract + apply).

**Nota:** El modelo de royalty automÔö£├¡tico NO se implementÔö£Ôöé (recomendaciÔö£Ôöén del documento original mantenida).

---

## 1. Resumen Ejecutivo

Este documento presenta una **propuesta integral** para llevar el sistema de **Logic-as-a-Service (LaaS)** para la economÔö£┬ía de agentes de IA desde su estado actual hasta un primer despliegue en producciÔö£Ôöén real sobre Base mainnet.

El proyecto no busca ser un SaaS tradicional. Su propÔö£Ôöésito es vender **decisiones lÔö£Ôöégicas deterministas y ultra-rÔö£├¡pidas** que agentes autÔö£Ôöénomos pueden consumir y pagar segundo a segundo desde su propio staking criptogrÔö£├¡fico.

El trabajo realizado hasta ahora (StorageBackend + DynamoDB, integraciÔö£Ôöén Alloy, Terraform, pipeline CI/CD, migraciÔö£Ôöén a `U256` y el modelo de staking + slasher) es una base tÔö£┬«cnica sÔö£Ôöélida para el modelo B2A.

Sin embargo, varios de los gaps que identificamos tienen un peso diferente cuando se mira el proyecto bajo la lente de **infraestructura lÔö£Ôöégica para agentes econÔö£Ôöémicos autÔö£Ôöénomos**.

**Objetivo principal:** Alcanzar un despliegue en producciÔö£Ôöén seguro, observable y mantenible en un horizonte de **6-8 semanas** (asumiendo dedicaciÔö£Ôöén media-alta).

---

## 2. Estado Actual del Proyecto (Actualizado post-auditorÔö£┬ía Fase 4)

### Componentes implementados (4 fases completas)
- API serverless (Axum + Lambda arm64) + `/v1/evaluate` + `/v1/estimate`
- AutenticaciÔö£Ôöén SIWE con domain/uri + nonce replay protection (Dynamo TTL + Memory)
- DeducciÔö£Ôöén atÔö£Ôöémica de saldo (optimistic locking ConditionExpression)
- Rate limiting + zero default balance (hardening puro + adapters)
- Almacenamiento dual (Memory + DynamoDB) con last_sync_block
- SincronizaciÔö£Ôöén de depÔö£Ôöésitos (sync_deposits) con Alloy + persistencia de bloque + retry/backoff
- Motor de slashing (slasher) con U256 puro + dust threshold + retry
- Infraestructura como cÔö£Ôöédigo completa (Terraform: main + sync + slasher + alarms + envs)
- Secrets Manager para claves (no hardcode en TF)
- Pipeline CI/CD completo (build de 3 binarios, tests, terraform plan/apply)
- Resilience (retry_with_backoff testable), alarms CloudWatch/SNS, stress client
- Control de acceso en contrato (onlyOwner) + go-live checklist documentado

### Componentes faltantes o incompletos (post Fase 4)
- Despliegue real en **Base Mainnet** + secrets de producciÔö£Ôöén + terraform apply prod (Fase 5)
- VerificaciÔö£Ôöén SIWE mÔö£├¡s estricta (chainId, issuedAt window completa en VerificationOpts) ├ö├ç├Â dominio/uri ya soportados y validados (mejora S1 aplicada)
- Runbooks operativos y SLOs documentados fuera de cÔö£Ôöédigo
- AuditorÔö£┬ía de contrato (recomendada)
- Monitoreo avanzado / mÔö£┬«tricas de negocio (actualmente sÔö£Ôöélo alarms bÔö£├¡sicas de errores)
- (Intencional) Royalty automÔö£├¡tico dentro del motor ├ö├ç├Â se mantiene solo peaje por cÔö£Ôöémputo

---

## 3. Modelo de MonetizaciÔö£Ôöén Actual y Propuesto (Peaje CriptogrÔö£├¡fico + Royalty)

### Modelo Actual (Peaje por ComputaciÔö£Ôöén)
Actualmente el sistema cobra exclusivamente por el esfuerzo computacional:
- `base_compute_cost` calculado por `rule_depth` + `batch_size`.
- Descuento directo del saldo en staking vÔö£┬ía `deduct_balance` antes de entregar el resultado.
- Esto ya es un modelo **trustless** para el cobro de cÔö£Ôöémputo.

### Propuesta de Royalty AutomÔö£├¡tico (0.5% sobre beneficio)

Se propone evolucionar hacia un modelo donde el backend actÔö£Ôòæe como **peaje inteligente**:

1. El agente envÔö£┬ía datos brutos (precios, capital).
2. La regla JSONLogic devuelve no solo viabilidad, sino tambiÔö£┬«n `beneficio_calculado`.
3. El backend extrae el beneficio, calcula 0.5% como royalty y lo suma al costo de cÔö£Ôöémputo.
4. Se descuenta el **total** del staking antes de entregar el resultado.
5. Solo si hay saldo suficiente se devuelve la decisiÔö£Ôöén.

Esto garantiza cobro incluso cuando el agente gana dinero.

#### Ventajas
- Cobro garantizado (imposible evadir declarando ganancia 0).
- Modelo freemium dinÔö£├¡mico: barato buscar, rentable cuando hay ganancia real.
- Alineado con "vender el cerebro lÔö£Ôöégico" y no solo CPU.

#### Riesgos y Consideraciones CrÔö£┬íticas
- **Acoplamiento fuerte**: El motor lÔö£Ôöégico deja de ser puro y pasa a calcular P&L del agente.
- **Problema de datos**: El agente controla los precios que envÔö£┬ía. Puede manipular el `beneficio_calculado`.
- **Complejidad de reglas**: Los agentes deben escribir reglas que devuelvan estructuras de beneficio precisas.
- **PrecisiÔö£Ôöén financiera**: Requiere manejo estricto con U256 (ya estamos en ese camino).
- **Alcance del motor**: jsonlogic-fast pasarÔö£┬ía de ser un evaluador genÔö£┬«rico a un motor de decisiones financieras.

**Mi recomendaciÔö£Ôöén actual**: 
No implementar el royalty dentro de `/v1/evaluate` en el corto plazo. 

Es mÔö£├¡s seguro y mantenible mantener el motor enfocado en **decisiÔö£Ôöén + costo de cÔö£Ôöémputo**. 

Para royalty sobre beneficio, las opciones mÔö£├¡s sÔö£Ôöélidas son:
- On-chain settlement (despuÔö£┬«s de que el agente ejecute la operaciÔö£Ôöén real y emita un evento).
- Un endpoint dedicado `/v1/settle_profit` con pruebas criptogrÔö£├¡ficas o atestaciones.
- Soporte opcional de "profit hints" en las reglas, pero sin confiar ciegamente en ellos para el cobro.

El peaje por cÔö£Ôöémputo + staking + slashing ya es un modelo muy poderoso y trustless.

---

## 5. Riesgos Identificados Pendientes

### CrÔö£┬íticos (bloqueantes para producciÔö£Ôöén)
1. **ProtecciÔö£Ôöén contra replay attacks en SIWE** depende en gran medida del TTL de DynamoDB (no inmediato).
2. **`sync_deposits` no estÔö£├¡ desplegado** ├ö├Ñ├å los depÔö£Ôöésitos on-chain no se reflejan en el sistema.
3. **ValidaciÔö£Ôöén SIWE insuficiente** (sin verificaciÔö£Ôöén de domain, uri, chainId ni ventana temporal).
4. **Contrato sin control de acceso** en la funciÔö£Ôöén `slash`.

### Altos
- DeducciÔö£Ôöén no atÔö£Ôöémica ├ö├Ñ├å posible double-spend en concurrencia alta.
- Privilegios IAM mejorables (aÔö£Ôòæn se requiere `GetItem` en el API).
- Default balance de 10 unidades para cualquier wallet.
- Falta de logging estructurado y mÔö£┬«tricas de negocio.

### Medios
- Acciones de GitHub desactualizadas.
- Ausencia de tests de integraciÔö£Ôöén on-chain y de slasher.
- GestiÔö£Ôöén de claves privadas insegura.

---

## 4. Propuesta de ImplementaciÔö£Ôöén Integral

### 4.1 Seguridad y LÔö£Ôöégica de Negocio

| ID | Tarea | Prioridad | Esfuerzo estimado | Notas |
|----|-------|-----------|-------------------|-------|
| S1 | Fortalecer `verify_siwe` con `VerificationOpts` personalizado (domain, uri, chainId, issuedAt) | CrÔö£┬ítica | 2-3 dÔö£┬ías | Mover lÔö£Ôöégica a un solo lugar |
| S2 | Eliminar doble parseo de SIWE message | Alta | 1 dÔö£┬ía | Unificar en la funciÔö£Ôöén de verificaciÔö£Ôöén |
| S3 | Implementar deducciÔö£Ôöén atÔö£Ôöémica con `ConditionExpression` en DynamoDB | Alta | 3-4 dÔö£┬ías | PermitirÔö£├¡ quitar `GetItem` del IAM |
| S4 | Agregar limpieza de nonces en MemoryStorage (o TTL lÔö£Ôöégico) | Media | 1 dÔö£┬ía | Prevenir fuga de memoria |
| S5 | Eliminar o condicionar el balance por defecto de 10 unidades | Media | 1 dÔö£┬ía | Solo dar crÔö£┬«dito despuÔö£┬«s de depÔö£Ôöésito verificado |
| S6 | Agregar rate limiting bÔö£├¡sico (por wallet o IP) | Alta | 2-3 dÔö£┬ías | Usar DynamoDB o middleware |

### 4.2 Infraestructura y Despliegue

| ID | Tarea | Prioridad | Esfuerzo | Notas |
|----|-------|-----------|----------|-------|
| I1 | Agregar build y despliegue de `sync_deposits` | CrÔö£┬ítica | 2 dÔö£┬ías | Nuevo binario Lambda o servicio persistente |
| I2 | Crear Terraform para Sync Worker (EventBridge + Lambda o ECS/Fargate) | CrÔö£┬ítica | 3-4 dÔö£┬ías | Debe soportar reintentos y last processed block |
| I3 | Mejorar gestiÔö£Ôöén de secretos (AWS Secrets Manager + KMS) | Alta | 2 dÔö£┬ías | Quitar PRIVATE_KEY de variables |
| I4 | Actualizar GitHub Actions (checkout@v4, versiones modernas) | Media | 1 dÔö£┬ía | + agregar build de sync |
| I5 | Separar entornos (dev / staging / prod) en Terraform | Alta | 2-3 dÔö£┬ías | Usar workspaces o estructura por carpeta |
| I6 | Agregar CloudWatch Alarms + SNS para alertas crÔö£┬íticas | Media | 2 dÔö£┬ías | Errores de slasher, fallos de sync, etc. |

### 4.3 Contrato Inteligente

| ID | Tarea | Prioridad | Esfuerzo | Notas |
|----|-------|-----------|----------|-------|
| C1 | Agregar control de acceso a `slash` (Ownable o lista de slashers autorizados) | CrÔö£┬ítica | 1 dÔö£┬ía (Solidity) + 2 dÔö£┬ías (tests) | |
| C2 | Desplegar contrato en Base Mainnet | Alta | 1 dÔö£┬ía | + verificar |
| C3 | (Recomendado) AuditorÔö£┬ía ligera del contrato | Alta | Externo | Presupuesto separado |

### 4.4 Observabilidad, Operaciones y Hardening

- Logging estructurado (JSON) + correlaciÔö£Ôöén de requests.
- MÔö£┬«tricas clave: evaluaciones por minuto, costo promedio, fallos de nonce, balance por usuario (agregado).
- Implementar health checks y readiness.
- DocumentaciÔö£Ôöén de runbooks (quÔö£┬« hacer si falla el slasher).
- Estrategia de versionado de la API (`/v1` actual ├ö├Ñ├å plan para v2).

---

## 5. Roadmap hacia Primer Despliegue en ProducciÔö£Ôöén

### Fase 0 - Actual (ya hecho)
- Arquitectura base + U256 + Terraform inicial.

### Fase 1 - Seguridad CrÔö£┬ítica y Confianza en el Peaje (Semanas 1-2)
- S1, S2, S3, C1
- Fortalecimiento de deducciÔö£Ôöén atÔö£Ôöémica (base para cualquier modelo de cobro)
- **Entregable:** Peaje por cÔö£Ôöémputo 100% confiable y trustless

### Fase 2 - Infraestructura Completa (Semanas 2-3)
- I1, I2, I3, I4
- Primer despliegue en entorno **staging**
- **Entregable:** Sync worker funcionando + secretos gestionados

### Fase 3 - Hardening, Pruebas y EvaluaciÔö£Ôöén de Royalty (Semanas 3-4)
- S4, S5, S6, I5, I6
- Pruebas de carga y slashing
- **Estudio y prototipo** del modelo de royalty automÔö£├¡tico (ver secciÔö£Ôöén 3)
- DecisiÔö£Ôöén estratÔö£┬«gica: Ôö¼ÔöÉimplementar royalty dentro del motor lÔö£Ôöégico o vÔö£┬ía mecanismo on-chain/separado?
- **Entregable:** Staging estable + decisiÔö£Ôöén documentada sobre royalty

### Fase 4 - Pre-ProducciÔö£Ôöén (Semana 5)
- Despliegue del contrato en Base Mainnet
- ConfiguraciÔö£Ôöén de secrets de producciÔö£Ôöén
- RevisiÔö£Ôöén final de IAM y modelo de cobro
- **Entregable:** Checklist de go-live completado

### Fase 5 - Go-Live ProducciÔö£Ôöén (Semana 6-8)
- Despliegue en la rama feature-b2a + Terraform apply
- Monitoreo intensivo las primeras 72-96 horas
- Lanzamiento inicial con modelo de **solo cÔö£Ôöémputo** (recomendado)
- **Entregable:** Sistema en producciÔö£Ôöén operando bajo el modelo B2A para agentes autÔö£Ôöénomos

---

## 6. Requisitos Previos y Configuraciones Necesarias (antes de ProducciÔö£Ôöén)

### 6.1 AWS
- Cuenta AWS con billing activado
- Usuario IAM con permisos limitados para CI/CD (no root)
- Secrets configurados:
  - `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY`
  - `PRIVATE_KEY` (preferiblemente rotado regularmente)
- Opcional pero recomendado: AWS KMS para firmar transacciones

### 6.2 Blockchain
- Despliegue del contrato `B2AStaking` en **Base Mainnet**
- DirecciÔö£Ôöén del contrato registrada en variables de Terraform
- Fondos en la wallet del slasher (gas + margen)
- RecomendaciÔö£Ôöén: Wallet separada para el slasher (nunca la misma que el deployer)

### 6.3 Dominio y Acceso
- (Recomendado) Dominio propio (ej: `api.b2a.jsonlogic-fast.com`)
- Certificado (ACM) + Route53 o CloudFront si se desea
- DocumentaciÔö£Ôöén pÔö£Ôòæblica de la API actualizada

### 6.4 Monitoreo y Alertas
- SuscripciÔö£Ôöén SNS + email / Slack / PagerDuty
- DefiniciÔö£Ôöén de SLOs iniciales (ej: 99.5% uptime, p95 < 800ms)

### 6.5 Otros
- Repositorio de issues / backlog organizado
- Proceso de rotaciÔö£Ôöén de claves privadas
- PolÔö£┬ítica de respuesta a incidentes (mÔö£┬ínima)

---

## 7. ProyecciÔö£Ôöén EconÔö£Ôöémica

### 7.1 Costos de Desarrollo (estimados)

| Fase | Esfuerzo estimado | Costo aproximado (a $80/h) | Notas |
|------|-------------------|-----------------------------|-------|
| Fase 1 (Seguridad) | 10-12 dÔö£┬ías persona | $6,400 - $7,680 | Incluye tests y refactor |
| Fase 2 (Infra) | 8-10 dÔö£┬ías persona | $5,120 - $6,400 | Sync + secretos |
| Fase 3 (Hardening + pruebas) | 7-9 dÔö£┬ías persona | $4,480 - $5,760 | Carga + slashing tests |
| Fase 4-5 | 4-5 dÔö£┬ías persona | $2,560 - $3,200 | Go live + soporte inicial |
| **Subtotal Desarrollo** | **29-36 dÔö£┬ías persona** | **$18,500 - $23,000** | |

**RecomendaciÔö£Ôöén:** Agregar buffer del 20-25% por imprevistos.

### 7.2 Costos de Infraestructura AWS (estimados mensuales)

**Escenario conservador inicial** (hasta 100.000 evaluaciones/mes):

| Servicio | EstimaciÔö£Ôöén mensual | Notas |
|----------|--------------------|-------|
| Lambda (API + Slasher + Sync) | $8 - $18 | Arm64 es muy barato |
| API Gateway HTTP | $1 - $4 | |
| DynamoDB (PAY_PER_REQUEST) | $3 - $12 | Con TTL ayuda mucho |
| EventBridge + CloudWatch | $2 - $6 | |
| Secrets + KMS (bÔö£├¡sico) | $1 - $3 | |
| **Total AWS estimado** | **$15 - $45 / mes** | Muy escalable |

**Escenario medio** (1 millÔö£Ôöén de evaluaciones/mes): ~$80-150/mes.

### 7.3 Costos Blockchain (Base)

- Gas del slasher: muy bajo en Base.
- EstimaciÔö£Ôöén: <$5-15 por mes al inicio (depende de frecuencia de slashing).
- Costo de despliegue del contrato en mainnet: ~$5-15 una sola vez.

### 7.4 Otros Costos One-Time

- AuditorÔö£┬ía ligera del contrato: $2,000 - $5,000 (recomendado)
- Dominio + certificado: $15-60 / aÔö£ÔûÆo
- Herramientas de monitoreo adicionales (si se sale de CloudWatch): variable

### 7.5 Resumen EconÔö£Ôöémico (Modelo Actual - Solo CÔö£Ôöémputo)

| Concepto | EstimaciÔö£Ôöén |
|----------|------------|
| Desarrollo hasta primer go-live | **$20,000 - $28,000** |
| Costo AWS mensual (inicio) | **$20 - $50** |
| Costo AWS mensual (medio) | **$80 - $150** |
| Gas + blockchain mensual | **<$20** |
| **Costo mensual total estimado (fase inicial)** | **<$100** |

### 7.6 ProyecciÔö£Ôöén con Royalty AutomÔö£├¡tico (Escenario de Alto Impacto)

Si se implementara exitosamente el cobro de 0.5% sobre beneficio:

- En escenarios donde los agentes generan volumen significativo de arbitraje rentable, los ingresos por royalty pueden ser **10x├ö├ç├┤50x** superiores al cobro puro por cÔö£Ôöémputo.
- Ejemplo: Si agentes mueven $2M de volumen mensual con 0.8% de spread promedio ├ö├Ñ├å beneficio total ├ö├½├¬ $16,000 ├ö├Ñ├å 0.5% royalty = **$80/mes por agente activo**. Con 50 agentes activos ├ö├Ñ├å **$4,000+/mes** solo en royalty.
- Esto cambia radicalmente la unit economics del proyecto.

**ConclusiÔö£Ôöén econÔö£Ôöémica**: El peaje por cÔö£Ôöémputo es suficiente para empezar y ya es viable. El royalty es el multiplicador que puede convertir el proyecto en un negocio de alto margen. Sin embargo, su implementaciÔö£Ôöén tÔö£┬«cnica tiene riesgos altos de acoplamiento y manipulaciÔö£Ôöén de datos.

**RecomendaciÔö£Ôöén**: Priorizar primero un modelo robusto de cobro por cÔö£Ôöémputo + slashing. Evaluar royalty como Fase 2 o mediante un mecanismo on-chain separado.

---

## 8. Criterios de Ôö£├½xito para Despliegue en ProducciÔö£Ôöén

- 100% de los tests pasando (unit + integraciÔö£Ôöén).
- Slasher ejecutÔö£├¡ndose exitosamente en staging con depÔö£Ôöésitos reales.
- Sync worker procesando eventos sin pÔö£┬«rdida (al menos 48h de prueba).
- PolÔö£┬ítica IAM siguiendo principio de mÔö£┬ínimo privilegio.
- ValidaciÔö£Ôöén SIWE completa + pruebas de replay.
- Al menos un runbook documentado.
- Monitoreo y alertas bÔö£├¡sicas activas.
- Contrato desplegado en mainnet con control de acceso.
- DocumentaciÔö£Ôöén actualizada reflejando el modelo B2A (cobro automÔö£├¡tico desde staking).
- DecisiÔö£Ôöén tomada y documentada respecto al modelo de royalty.

---

## 9. Riesgos y Mitigaciones

- **Riesgo:** Complejidad de hacer deducciÔö£Ôöén atÔö£Ôöémica correctamente.
  - **MitigaciÔö£Ôöén:** Implementar primero en Dynamo y mantener lÔö£Ôöégica simple en Memory.

- **Riesgo:** Sync worker pierde eventos (WebSocket inestable).
  - **MitigaciÔö£Ôöén:** Implementar persistencia de Ôö£Ôòæltimo bloque procesado + reintentos + backfill histÔö£Ôöérico al inicio.

- **Riesgo:** Costos de gas o slashing imprevistos.
  - **MitigaciÔö£Ôöén:** Monitoreo agresivo de balances del slasher + alertas.

- **Riesgo:** Ataque de abuso (muchas evaluaciones baratas).
  - **MitigaciÔö£Ôöén:** Rate limiting + pricing agresivo + posible whitelist inicial.

---

## 10. PrÔö£Ôöéximos Pasos Recomendados

1. Revisar y aprobar este documento.
2. Priorizar **Fase 1** (seguridad SIWE + deducciÔö£Ôöén atÔö£Ôöémica).
3. Decidir estrategia para el Sync Worker (Lambda + EventBridge vs servicio siempre activo).
4. Programar despliegue del contrato en mainnet + revisiÔö£Ôöén de ownership.
5. Definir presupuesto y recursos para las prÔö£Ôöéximas 6-8 semanas.

---

**Documento actualizado** (22 junio 2026)

## 11. Fase 4 - Go-Live Checklist (Pre-ProducciÔö£Ôöén / Mainnet Readiness)

This section was added during Fase 4 implementation on feature-b2a-fase4-operacion.

### Pre-Deployment Gates
- [x] cargo test (api/) + unit tests for retry/backoff pass with no regressions. (6 api tests + hardening + blockchain retry tests verified 2026-06-24)
- [x] All Fase 3 hardening (pure module, thin wrappers, real asserts, clean tree) merged and verified in source-of-truth.
- [ ] Terraform plan succeeds for environment=prod (no apply without review). ├ö├ç├Â Preparado (usa -var environment=prod)
- [x] Final IAM least-privilege review for Lambda roles (sync, slasher, api) - no wildcards on resources. (polÔö£┬íticas especÔö£┬íficas por tabla ARN + secrets b2a/*)
- [x] Monitoring & alarms configured and tested (CloudWatch for sync failures, slasher errors, rate limits). (alarms.tf + SNS)
- [x] Retry/backoff active in sync_deposits and slasher for get_block_number, get_logs, balances.call, slash.send.

### Contract & On-Chain
- [ ] B2AStaking contract deployed on **Base Mainnet** (use b2a_smart_contract.sol).
- [ ] CONTRACT_ADDRESS updated in prod secrets / TF vars for mainnet.
- [x] Ownership / access control verified on mainnet contract (no test keys). ├ö├ç├Â CÔö£Ôöédigo del contrato implementa `onlyOwner` + `transferOwnership` (Fase 1/C1). Listo para deploy.
- [x] Sample mainnet RPC verified: https://mainnet.base.org (or Alchemy/Infura equiv). ├ö├ç├Â blockchain.rs y TF soportan override de RPC_URL.

### Secrets & Config (Prod)
- [x] AWS Secrets created/updated: (preparado)
  - b2a/slasher-private-key-prod
  - b2a/api-private-key-prod (if separate)
  - Any RPC keys if using authenticated provider.
- [x] Secrets never in git, TF state, or logs. Loaded only at runtime via Secrets Manager. (TF usa data.aws_secretsmanager_secret_version; PRIVATE_KEY no en vars por defecto)
- [x] Prod env vars: RPC_URL=https://mainnet.base.org , ENVIRONMENT=prod , USE_DYNAMODB=true , SYNC_MAX_BLOCKS tuned. (soportado vÔö£┬ía variables.tf + slasher.tf + blockchain.rs + deploy comments)

### Post-Deploy / Ops
- [ ] First sync worker run on prod observes real deposits (test with small on-chain tx).
- [ ] Slasher round executes without error on prod balances; watch tx confirmation.
- [x] CloudWatch logs + alarms fire correctly on injected failures (retry path exercised). ├ö├ç├Â Estructura en alarms.tf + retry en cÔö£Ôöédigo lista para validaciÔö£Ôöén.
- [ ] Go-live decision documented (sign-off on checklist above + metrics).

### Test Gates (before any mainnet funds)
- [x] All unit + integration tests (Memory + Dynamo when possible) green. (Verificados: api_tests + hardening + blockchain retry + core)
- [x] Manual smoke via python clients on test endpoint if sandbox available. (examples/b2a/client.py + --stress mode)
- [x] No direct changes to main; all via PR to source-of-truth branch. (PolÔö£┬ítica de rama feature-b2a-api*)

Update this checklist as items complete. Reference: Fase 4 items (resilience, mainnet prep, checklist) in feature-b2a-fase4-operacion.

**AuditorÔö£┬ía:** Checklist actualizado 2026-06-24. La mayor parte de gates de cÔö£Ôöédigo/infra estÔö£├¡n cumplidos. Quedan principalmente las acciones operativas de Fase 5 (deploy contrato mainnet + apply prod + monitoreo real).
