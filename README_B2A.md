# API Serverless jsonlogic-fast B2A

Este directorio contiene la implementación de la API B2A (Business-to-Agent) de `jsonlogic-fast`.

## Descripción del proyecto

`jsonlogic-fast` B2A es un motor ultrarrápido para evaluar reglas dinámicas en formato JSON, diseñado específicamente para agentes autónomos e inteligencias artificiales. Permite a los agentes delegar la evaluación lógica compleja, la toma de decisiones y el procesamiento por lotes de reglas de negocio en un entorno sin servidor y de alto rendimiento. Es ideal para que los agentes validen transacciones, analicen permisos o apliquen reglas condicionales sin sobrecargar su propio entorno de ejecución.

## Arquitectura

La solución B2A se compone de los siguientes elementos:
- **API Server**: Un servidor HTTP asíncrono en Rust (basado en Axum o desplegable como AWS Lambda) que expone los endpoints de estimación de costos y evaluación de reglas.
- **Autenticación SIWE**: Las solicitudes se autentican mediante el estándar *Sign-In with Ethereum* (EIP-4361). Los agentes firman las peticiones usando sus billeteras criptográficas.
- **Pricing Engine**: Un módulo de precios dinámico que calcula el costo de una evaluación en función de su profundidad lógica (rule_depth) y la cantidad de datos (batch_size).
- **Smart Contract (Mock)**: Un contrato inteligente simulado (`b2a_smart_contract.sol`) para el staking y la facturación, donde se cobra por cada evaluación descontando saldo de la billetera del agente.

## Endpoints

### `POST /v1/estimate`
Calcula el costo estimado de ejecutar una evaluación sin consumirla.
- **Payload esperado (JSON)**:
  - `rule_depth` (entero): La profundidad máxima estimada de la regla (ej. 1).
  - `batch_size` (entero): El número de contextos/elementos de datos a evaluar (ej. 100).
- **Respuesta (JSON)**:
  - `estimated_cost` (flotante): El costo estimado de la operación.

### `POST /v1/evaluate`
Ejecuta la evaluación de una regla contra unos datos y descuenta el costo del saldo del remitente. Requiere autenticación válida.
- **Payload esperado (JSON)**:
  - `message` (string): El mensaje SIWE completo que fue firmado.
  - `signature` (string): La firma criptográfica generada por la wallet del agente.
  - `rule` (JSON object): La regla en formato JsonLogic a evaluar.
  - `data` (JSON object/array): El contexto o los datos sobre los que se evaluará la regla. Si es un arreglo, determina el `batch_size`.
- **Respuesta (JSON)**:
  - `result` (JSON object/array): El resultado de la evaluación.
  - `cost` (flotante): El costo exacto que se dedujo del saldo.

## Modelo de precios

El costo se calcula dinámicamente utilizando una tarifa base por cada evaluación y un recargo según la profundidad de la regla:
`costo = tarifa_base * (1.0 + (rule_depth * 0.1)) * batch_size`
Donde la tarifa base típica actual es `0.0001` USDC (o unidades de saldo) por evaluación. Es decir, una regla con profundidad de 1 sobre un único dato costará `0.00011`.

## Autenticación

Todas las solicitudes a `/v1/evaluate` deben firmarse utilizando el estándar EIP-4361 (Sign-In with Ethereum). El cliente genera un mensaje SIWE estándar, lo firma con su clave privada y envía el mensaje original y la firma al endpoint.

### Ejemplo de firma con `ethers.js`

```javascript
import { ethers } from 'ethers';
import { SiweMessage } from 'siwe';

async function signRequest() {
    const wallet = new ethers.Wallet('0x...tu_clave_privada...');
    const address = await wallet.getAddress();

    const domain = 'api.jsonlogic-fast.local';
    const origin = 'https://api.jsonlogic-fast.local';

    const message = new SiweMessage({
        domain,
        address,
        statement: 'Sign in to jsonlogic-fast B2A API',
        uri: origin,
        version: '1',
        chainId: 1
    });

    const messageToSign = message.prepareMessage();
    const signature = await wallet.signMessage(messageToSign);

    return { message: messageToSign, signature };
}
```

## Ejemplos prácticos

### 1. Estimación mediante cURL
```bash
curl -X POST http://localhost:3000/v1/estimate \
     -H "Content-Type: application/json" \
     -d '{"rule_depth": 1, "batch_size": 10}'
```

### 2. Evaluación mediante JavaScript (Node.js/Axios)
```javascript
const axios = require('axios');
// Asume que obtienes `message` y `signature` del ejemplo de ethers.js

const payload = {
    message: messageToSign,
    signature: signature,
    rule: { "==": [1, 1] },
    data: {}
};

axios.post('http://localhost:3000/v1/evaluate', payload)
    .then(response => console.log(response.data))
    .catch(error => console.error(error.response.data));
```

### 3. Evaluación mediante Python (Requests)
```python
import requests

# Genera el mensaje SIWE y la firma usando una librería equivalente en Python como 'eth-account' y 'siwe'
payload = {
    "message": "...",      # Mensaje SIWE generado
    "signature": "0x...",  # Firma del agente
    "rule": { "and": [{ ">": [{"var": "temp"}, 100]}, { "==": [{"var": "status"}, "active"] }] },
    "data": { "temp": 150, "status": "active" }
}

response = requests.post("http://localhost:3000/v1/evaluate", json=payload)
print(response.json())
```

## Despliegue

### Entorno local
Para ejecutar el servidor localmente con Axum, usa el siguiente comando en la raíz del repositorio o dentro de la carpeta `api`:
```bash
cargo run -p api --bin api_server
```
El servidor escuchará en `0.0.0.0:3000`.

### Despliegue en AWS Lambda
Para compilar y desplegar el servicio como una función serverless en AWS Lambda, se utiliza `cargo lambda`:
```bash
cargo lambda build --release --arm64
cargo lambda deploy
```
