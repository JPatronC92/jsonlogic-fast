# API B2A jsonlogic-fast

La API B2A expone `jsonlogic-fast` como un MVP público controlado: el core sigue siendo open source y gratis, mientras que la API hosted usa API keys con Free Tier limitado. SIWE se mantiene disponible como modo avanzado/experimental para flujos con wallet y balance on-chain/off-chain.

## Private Beta Positioning

La API B2A se encuentra actualmente en **Private Beta**. Aunque es funcional y tiene salvaguardas de producción, el modelo de negocio, cuotas de tiers, latencias y configuraciones de CORS pueden cambiar en el futuro.

## Free Tier hosted

- Flujo principal: `Authorization: Bearer <api_key>`.
- Límite mensual Free Tier: **1000 evaluaciones/mes** por API key.
- Rate limit Free Tier: **10 requests/minuto** por API key.
- Las API keys no se guardan en texto plano; el servicio almacena y compara hashes SHA-256 en atributos planos de DynamoDB.
- Cuando se supera el límite mensual, `/v1/evaluate` responde `402 Payment Required`.

### Generación de API Keys

Actualmente, no existe un endpoint público sin autenticación para generar API keys. Esto se hace de manera administrativa a través de la CLI integrada en el repositorio.

Ejemplo:
```bash
cargo run -p api --bin api_key_admin -- create --environment dev --owner julio@example.com --plan free
```
Esto generará una clave aleatoria (e.g. `b2a_beta_<base64_hash>`), y la mostrará por única vez.
El hash se guardará atómicamente en DynamoDB para seguimiento mensual.

Las API keys se pueden revocar (desactivar):
```bash
cargo run -p api --bin api_key_admin -- revoke --environment dev --hash <api_key_hash>
```

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

#### Errores estandarizados
Los endpoints ahora manejan un "envelope" estructurado para errores de uso en `Evaluate` y `Usage`:
```json
{
  "error": {
    "code": "monthly_limit_exceeded",
    "message": "Monthly free tier limit exceeded",
    "request_id": "req-123"
  }
}
```

Principales códigos de estado de error:
- `401 Unauthorized`: API key faltante o invalida (`missing_bearer_token`, `invalid_api_key`) o SIWE signature erronea.
- `402 Payment Required`: La API Key llegó a su límite mensual del tier. (`monthly_limit_exceeded`)
- `429 Too Many Requests`: Ha excedido el límite de evaluaciones/rate limits. (`rate_limit_exceeded`)
- `400 Bad Request`: Falla parseo de JSONLogic u otro problema en la peticion de evaluacion. (`invalid_rule`)
- `500 Internal Server Error`: Falla temporal del DB, mal parseo de evaluacion.

## Production Guardrails
En entorno de producción (`ENVIRONMENT=prod`):
- Los lambdas `slasher` y `sync_deposits` van a fallar y apagarse de inmediato si se usan con `USE_DYNAMODB=false` (Memoria temporal).
- `blockchain` requerirá que se encuentre una llave privada de SecretsManager verdadera; de lo contrario tirará un panic, denegando fallbacks a llaves dummy locales.

