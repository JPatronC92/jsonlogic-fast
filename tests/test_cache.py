from unittest.mock import patch

from src.core.cache import SimpleTTLCache


def test_set_and_get():
    cache = SimpleTTLCache()
    cache.set("key", "value")
    assert cache.get("key") == "value"


def test_get_nonexistent():
    cache = SimpleTTLCache()
    assert cache.get("nonexistent") is None


def test_expiration():
    cache = SimpleTTLCache(ttl_seconds=10)
    with patch("time.time") as mock_time:
        mock_time.return_value = 1000
        cache.set("key", "value")

        # Still valid at 1009
        mock_time.return_value = 1009
        assert cache.get("key") == "value"

        # Expired at 1010
        mock_time.return_value = 1010
        assert cache.get("key") is None
        assert "key" not in cache._cache


def test_overwrite():
    cache = SimpleTTLCache(ttl_seconds=10)
    with patch("time.time") as mock_time:
        mock_time.return_value = 1000
        cache.set("key", "value1")

        mock_time.return_value = 1005
        cache.set("key", "value2")

        # Should expire at 1015 now, not 1010
        mock_time.return_value = 1012
        assert cache.get("key") == "value2"


def test_clear():
    cache = SimpleTTLCache()
    cache.set("k1", "v1")
    cache.set("k2", "v2")
    cache.clear()
    assert cache.get("k1") is None
    assert cache.get("k2") is None
    assert len(cache._cache) == 0


def test_custom_ttl():
    cache = SimpleTTLCache(ttl_seconds=60)
    with patch("time.time") as mock_time:
        mock_time.return_value = 1000
        cache.set("key", "value")
        assert cache._cache["key"]["expires_at"] == 1060
