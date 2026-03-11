import time
from typing import Any, Dict, Optional


class SimpleTTLCache:
    def __init__(self, ttl_seconds: int = 300):
        self.ttl_seconds = ttl_seconds
        self._cache: Dict[str, Dict[str, Any]] = {}

    def set(self, key: str, value: Any) -> None:
        self._cache[key] = {
            "value": value,
            "expires_at": time.time() + self.ttl_seconds,
        }

    def get(self, key: str) -> Optional[Any]:
        if key in self._cache:
            item = self._cache[key]
            if time.time() < item["expires_at"]:
                return item["value"]
            else:
                del self._cache[key]
        return None

    def clear(self) -> None:
        self._cache.clear()
