class LexEngineError(Exception):
    """Base exception for Lex MX Engine."""
    pass

class VigenciaOverlapError(LexEngineError):
    """Raised when a version's validity period overlaps with an existing one."""
    pass

class UnidadNoEncontradaError(LexEngineError):
    """Raised when a structural unit cannot be found."""
    pass

class PatchError(LexEngineError):
    """Raised when a patch application fails."""
    pass
