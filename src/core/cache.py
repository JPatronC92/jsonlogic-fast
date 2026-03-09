import time
from typing import Any, Optional


class SimpleTTLCache:
    def __init__(self, ttl_seconds: int = 300):
        self.ttl_seconds = ttl_seconds
        self._cache: dict[str, dict[str, Any]] = {}

    def set(self, key: str, value: Any) -> None:
        self._cache[key] = {
            "value": value,
            "expires_at": time.time() + self.ttl_seconds,
        }

    def get(self, key: str) -> Optional[Any]:
        if key not in self._cache:
            return None

        entry = self._cache[key]
        if time.time() >= entry["expires_at"]:
            del self._cache[key]
            return None

        return entry["value"]

    def clear(self) -> None:
        self._cache.clear()
