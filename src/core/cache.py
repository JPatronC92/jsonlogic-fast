import time
from typing import Any, Dict, Optional

class SimpleTTLCache:
    """
    Caché en memoria súper rápida para evitar pegarle a Postgres o Redis
    por cada transacción del ERP.
    Útil para guardar temporalmente las reglas activas de un ámbito en un día específico.
    """
    def __init__(self, ttl_seconds: int = 300): # 5 minutos por defecto
        self.ttl = ttl_seconds
        self._cache: Dict[str, Dict[str, Any]] = {}

    def get(self, key: str) -> Optional[Any]:
        if key in self._cache:
            entry = self._cache[key]
            if time.time() < entry['expires_at']:
                return entry['value']
            else:
                del self._cache[key] # Expiró
        return None

    def set(self, key: str, value: Any):
        self._cache[key] = {
            'value': value,
            'expires_at': time.time() + self.ttl
        }

    def clear(self):
        self._cache.clear()

# Singleton global para la app
rule_cache = SimpleTTLCache(ttl_seconds=600) # 10 minutos
