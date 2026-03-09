import logging
import hashlib
import json
from typing import Dict, Any, List
from json_logic import jsonLogic
from jsonschema import validate, ValidationError
from src.domain.schemas.pricing import (
    CalculateFeeResponse,
    FeeBreakdown,
    BatchSimulateResponse,
)
from src.domain.models import PricingRuleVersion

try:
    import tempus_core

    RUST_CORE_AVAILABLE = True
except ImportError:
    RUST_CORE_AVAILABLE = False

logger = logging.getLogger("tempus.pricing_engine")


class PricingEngine:
    """
    El motor matemático determinista.
    Evalúa transacciones contra un set de reglas históricas (json-logic).
    """

    def simulate_batch(
        self,
        transacciones: List[Dict[str, Any]],
        reglas_activas: List[PricingRuleVersion],
    ) -> BatchSimulateResponse:
        """
        Simula el impacto de un esquema de precios sobre un lote masivo de transacciones.
        Retorna el P&L (Profit & Loss) agregado.
        """
        if RUST_CORE_AVAILABLE and transacciones:
            try:
                # 🦀 Vía Ultra Rápida (Rust Array Batch)
                valid_tx_strings = []
                valid_tx_amounts = []
                local_failed_count = 0
                local_success_count = 0

                for tx in transacciones:
                    try:
                        amt = float(tx.get("amount", 0.0))
                        if amt <= 0:
                            local_failed_count += 1
                            continue
                        valid_tx_amounts.append(amt)
                        valid_tx_strings.append(json.dumps(tx))
                        local_success_count += 1
                    except (ValueError, TypeError):
                        local_failed_count += 1

                total_vol = sum(valid_tx_amounts)
                fees_por_tx = [0.0] * local_success_count

                for regla_version in reglas_activas:
                    rule_str = json.dumps(regla_version.logica_json)
                    batch_fees = tempus_core.evaluate_batch(rule_str, valid_tx_strings)
                    for i, fee in enumerate(batch_fees):
                        fees_por_tx[i] += fee

                total_fees = sum(fees_por_tx)
                total_net = total_vol - total_fees

                return BatchSimulateResponse(
                    total_processed_volume=round(total_vol, 2),
                    total_fees_collected=round(total_fees, 2),
                    total_net_settlement=round(total_net, 2),
                    transactions_count=local_success_count,
                    failed_transactions=local_failed_count,
                )
            except Exception as e:
                logger.warning(
                    f"Error en fast-path Rust Batch, fallback a iteración: {e}"
                )

        # 🐍 Fallback: Iteración transaccional
        total_vol = 0.0
        total_fees = 0.0
        total_net = 0.0
        success_count = 0
        failed_count = 0

        for tx in transacciones:
            try:
                res = self.calculate(tx, reglas_activas)
                total_vol += res.base_amount
                total_fees += res.total_fees
                total_net += res.net_settlement
                success_count += 1
            except Exception as e:
                # En un Batch, no detenemos el proceso por una transacción fallida (ej. sin esquema)
                failed_count += 1
                logger.warning(f"Batch Sim. Tx Fallida: {str(e)}")

        return BatchSimulateResponse(
            total_processed_volume=round(total_vol, 2),
            total_fees_collected=round(total_fees, 2),
            total_net_settlement=round(total_net, 2),
            transactions_count=success_count,
            failed_transactions=failed_count,
        )

    def calculate(
        self, contexto_tx: Dict[str, Any], reglas_activas: List[PricingRuleVersion]
    ) -> CalculateFeeResponse:
        calculated_fees = []

        # 1. Validación de Esquema (Barrera Defensiva)
        for regla_version in reglas_activas:
            if regla_version.context_schema:
                try:
                    validate(
                        instance=contexto_tx,
                        schema=regla_version.context_schema.schema_json,
                    )
                except ValidationError as e:
                    logger.error(
                        f"Contexto inválido para regla {regla_version.rule.name}: {e.message}"
                    )
                    raise ValueError(
                        f"Payload malformado para {regla_version.rule.name}: {e.message}"
                    )

        # 2. Extracción Segura
        try:
            base_amount = float(contexto_tx.get("amount", 0.0))
        except (ValueError, TypeError):
            raise ValueError("El 'amount' base de la transacción debe ser numérico.")

        currency = contexto_tx.get("currency", "UNKNOWN")

        if base_amount <= 0:
            raise ValueError("El 'amount' base de la transacción debe ser mayor a 0.")

        total_fees = 0.0
        reglas_hashes = []

        # 3. Evaluación Matemática Determinista
        for regla_version in reglas_activas:
            try:
                if RUST_CORE_AVAILABLE:
                    try:
                        # 🦀 Vía Rápida (Rust Native)
                        rule_str = json.dumps(regla_version.logica_json)
                        ctx_str = json.dumps(contexto_tx)
                        fee_amount = tempus_core.evaluate_fee(rule_str, ctx_str)
                    except Exception as rust_err:
                        logger.warning(
                            f"Tempus Core (Rust) falló para regla {regla_version.rule.name}, fallback a Python: {rust_err}"
                        )
                        fee_amount = float(
                            jsonLogic(regla_version.logica_json, contexto_tx)
                        )
                else:
                    # 🐍 Vía Lenta (Python)
                    fee_amount = float(
                        jsonLogic(regla_version.logica_json, contexto_tx)
                    )

                if fee_amount > 0:
                    calculated_fees.append(
                        FeeBreakdown(
                            rule_id=str(regla_version.rule_uuid),
                            name=regla_version.rule.name,
                            amount=fee_amount,
                        )
                    )
                    total_fees += fee_amount
                    reglas_hashes.append(regla_version.hash_firma or "unverified")

            except Exception as e:
                logger.error(
                    f"Error ejecutando json-logic para rule {regla_version.rule_uuid}: {e}"
                )
                raise ValueError(
                    f"Error calculando fee para {regla_version.rule.name}: {str(e)}"
                )

        net_settlement = base_amount - total_fees

        # 2. Sello Criptográfico de Auditoría
        # Creamos un hash determinista de las reglas específicas que se ejecutaron
        # para demostrar irrefutablemente cómo se calculó este precio.
        audit_payload = json.dumps({"hashes": sorted(reglas_hashes)}).encode("utf-8")
        execution_hash = hashlib.sha256(audit_payload).hexdigest()

        return CalculateFeeResponse(
            base_amount=base_amount,
            calculated_fees=calculated_fees,
            total_fees=total_fees,
            net_settlement=net_settlement,
            currency=currency,
            cryptographic_hash=f"sha256:{execution_hash}",
        )
