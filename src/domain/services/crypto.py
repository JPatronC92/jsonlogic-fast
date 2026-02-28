import json
import hashlib
from datetime import datetime, timezone
from typing import Any, Dict

def _to_iso_utc(value: datetime | str | None) -> str | None:
    if value is None:
        return None
    if isinstance(value, str):
        return value
    if isinstance(value, datetime):
        if value.tzinfo is None:
            value = value.replace(tzinfo=timezone.utc)
        return value.astimezone(timezone.utc).isoformat().replace("+00:00", "Z")
    raise TypeError("Unsupported date type for canonicalization")

def canonicalize_payload(
    *,
    urn_global: str,
    vigencia_desde: datetime | str,
    vigencia_hasta: datetime | str | None,
    esquema_id: str,
    logica_json: Dict[str, Any],
) -> bytes:
    payload = {
        "urn_global": urn_global,
        "vigencia_desde": _to_iso_utc(vigencia_desde),
        "vigencia_hasta": _to_iso_utc(vigencia_hasta),
        "esquema_id": str(esquema_id),
        "logica_json": logica_json,
    }

    canonical_str = json.dumps(
        payload,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
    )

    return canonical_str.encode("utf-8")

def sha256_hash(canonical_bytes: bytes) -> str:
    return hashlib.sha256(canonical_bytes).hexdigest()
