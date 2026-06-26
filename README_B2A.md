# API B2A jsonlogic-fast

La API B2A expone `jsonlogic-fast` como un MVP público controlado: el core sigue siendo open source y gratis, mientras que la API hosted usa API keys con Free Tier limitado. SIWE se mantiene disponible como modo avanzado/experimental para flujos con wallet y balance on-chain/off-chain.

## Free Tier hosted

- Flujo principal: `Authorization: Bearer <api_key>`.
- Límite mensual Free Tier: **1000 evaluaciones/mes** por API key.
- Rate limit Free Tier: **10 requests/minuto** por API key.
- Las API keys no se guardan en texto plano; el servicio almacena y compara hashes SHA-256.
- Cuando se supera el límite mensual, `/v1/evaluate` responde `402 Payment Required`.

## Endpoints

### `GET /health`

Respuesta simple de health check:

```json
{"status":"ok","service":"jsonlogic-fast-b2a"}
```

### `POST /v1/estimate`

Calcula el costo estimado sin ejecutar la regla.

```json
{
  "rule_depth": 1,
  "batch_size": 10
}
```

Respuesta:

```json
{
  "estimated_cost": "1100000000000000"
}
```

`estimated_cost` se devuelve como **string decimal entero con 18 decimales implícitos**, no como float, para evitar pérdida de precisión.

### `GET /v1/usage`

Requiere API key mediante `Authorization: Bearer <api_key>` y devuelve uso mensual actual.

```bash
curl http://localhost:3000/v1/usage \
  -H "Authorization: Bearer $B2A_API_KEY"
```

Respuesta:

```json
{
  "owner": "agent@example.com",
  "plan": "free",
  "monthly_limit": 1000,
  "used_this_month": 12,
  "remaining_this_month": 988,
  "rate_limit_per_minute": 10
}
```

### `POST /v1/evaluate`

Evalúa una regla JSONLogic. Soporta dos modos de autenticación:

1. **API key (principal / Free Tier)**: enviar `Authorization: Bearer <api_key>`. No requiere `message` ni `signature`; consume uso mensual.
2. **SIWE (avanzado / experimental)**: enviar `message` y `signature`; mantiene nonce, rate limit por wallet, balance y descuento de saldo.

Ejemplo principal con API key:

```bash
curl -X POST http://localhost:3000/v1/evaluate \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $B2A_API_KEY" \
  -d '{
    "rule": {"+": [{"var":"a"}, {"var":"b"}]},
    "data": {"a": 10, "b": 20}
  }'
```

Respuesta:

```json
{
  "result": 30,
  "cost": "120000000000000"
}
```

`cost` se devuelve como **string decimal entero con 18 decimales implícitos**, no como float.

#### Batch real

Si `data` es un array, cada elemento se evalúa como contexto individual y el uso/costo se calcula con el número real de elementos:

```bash
curl -X POST http://localhost:3000/v1/evaluate \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $B2A_API_KEY" \
  -d '{
    "rule": {"var":"score"},
    "data": [{"score": 90}, {"score": 45}, {"score": 100}]
  }'
```

Respuesta esperada:

```json
{
  "result": [90, 45, 100],
  "cost": "330000000000000"
}
```

## SIWE avanzado/experimental

SIWE sigue disponible para agentes que necesitan firmar con wallet. El cliente genera un mensaje EIP-4361, lo firma y envía `message`, `signature`, `rule` y `data` a `/v1/evaluate`. El servidor valida dominio/URI/cadena configurables, registra nonces para prevenir replay, aplica rate limit y descuenta balance.

Variables relevantes:

- `SIWE_DOMAIN`
- `SIWE_URI`
- `SIWE_CHAIN_ID`
- `SIWE_MAX_AGE_SECS`
- `ENVIRONMENT`

En `ENVIRONMENT=prod`, `MemoryStorage` se rechaza: producción debe usar almacenamiento persistente.

## Despliegue

El workflow de deploy es manual (`workflow_dispatch`) y ejecuta build/tests/`terraform plan`. No hace `terraform apply` automático en push a ramas feature. El apply debe ejecutarse manualmente tras revisar el plan.

## Desarrollo local

```bash
cargo run -p api --bin api_server
```

Por defecto el servidor local puede usar `MemoryStorage` solo fuera de `ENVIRONMENT=prod`.
