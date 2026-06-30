# Release Checklist B2A

Antes de realizar el merge a la rama principal (deploy a producción/Private Beta), se **deben** ejecutar y aprobar todos los comandos siguientes de forma local y/o en el entorno CI para asegurar la estabilidad, seguridad, y auditoría técnica:

## 1. Verificación de Integridad de Código
```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
```

## 2. Pruebas Backend API (Rust)
```bash
cargo test -p api
cd core && cargo test --verbose
```

## 3. Pruebas Cross-Language Bindings
```bash
make test
make test-python
make test-wasm
```

## 4. Auditoría de Seguridad & Dependencias
```bash
cargo run --bin cargo-deny -- check
cargo audit
```
*Note: Validar que `deny.toml` contiene solo las vulnerabilidades y librerías que temporal o permanentemente se ignoran bajo justificación adecuada.*

## 5. Terraform Infrastructure Validation
```bash
terraform -chdir=terraform fmt -check
terraform -chdir=terraform validate
terraform -chdir=terraform plan -var="environment=dev"
```
*(No existe Terraform apply automático. Todo cambio estructural debe ser confirmado por un plan review y ejecutado manual o controladamente).*

---
**Recuerde**:
- Asegurarse que el CI Actions reporta status `Verde` (pasado) antes de fusionar.
- Confirmar que cualquier uso de base de datos/local cache temporal (`MemoryStorage`) está aislado para evitar uso en entorno Productivo (`ENVIRONMENT=prod`).
- Al desplegar producción de Lambda functions o binarios, la variables de entorno `ENVIRONMENT`, `USE_DYNAMODB`, y dependencias de AWS Credentials/Secrets Manager deben estar correctamente alimentadas.
