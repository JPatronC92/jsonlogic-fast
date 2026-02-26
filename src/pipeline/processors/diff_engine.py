from src.infrastructure.llm_client import LLMClient
from src.domain.schemas.patch import PatchCandidate
from src.domain.exceptions import LexEngineError

class DiffEngine:
    """
    Orchestrates the LLM to generate a PatchCandidate from raw text.
    """

    def __init__(self, llm_client: LLMClient):
        self.llm = llm_client

    async def generate_patch_candidate(self, raw_text: str, url_fuente: str = "Desconocida") -> PatchCandidate:
        """
        Takes raw text from a decree and returns a structured PatchCandidate.
        """
        try:
            patch = await self.llm.parse_decreto_to_patch(raw_text, url_fuente)
            return patch
        except Exception as e:
            # Wrap error for clearer domain context
            raise LexEngineError(f"Error parsing decree: {str(e)}") from e
