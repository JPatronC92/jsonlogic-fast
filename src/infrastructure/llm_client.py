import json
import re
from litellm import completion
from src.core.config import get_settings
from src.domain.schemas.patch import PatchCandidate
from src.domain.exceptions import LexEngineError

settings = get_settings()

class LLMParserError(LexEngineError):
    """Raised when the LLM fails to parse a decree."""
    pass

class LLMClient:
    """
    El 'Cerebro' de Lex MX Engine. 
    Usa LLMs para transformar texto legal crudo en parches estructurados.
    """

    SYSTEM_PROMPT = """
    Eres un experto en derecho fiscal y administrativo mexicano con precisión quirúrgica.
    Tu tarea es leer decretos publicados en el Diario Oficial de la Federación (DOF) 
    y extraer las modificaciones a leyes existentes en un formato JSON estructurado.

    REGLAS DE EXTRACCIÓN:
    1. Identifica la norma (ley/código) que se reforma. (Ej: Código Fiscal de la Federación)
    2. Identifica el artículo, fracción o párrafo específico modificado.
    3. Extrae el texto NUEVO tal cual aparece en el decreto.
    4. Determina la acción: 'NUEVA_VERSION', 'DEROGACION', 'ALTA_NUEVA'.
    5. Identifica la fecha de inicio de vigencia basándose en los artículos transitorios del decreto.
    6. Genera el JSON siguiendo EXACTAMENTE el esquema PatchCandidate.

    IMPORTANTE: No inventes datos. Si no hay una fecha de vigencia clara, asume el día siguiente a la publicación por defecto.
    """

    def __init__(self):
        self.model = settings.LLM_MODEL or "gpt-4o"
        self.api_key = settings.LLM_API_KEY

    async def parse_decreto_to_patch(self, texto_decreto: str, url_fuente: str) -> PatchCandidate:
        """
        Envía el texto del decreto al LLM y devuelve un PatchCandidate validado.
        """
        print(f"🧠 Enviando decreto a LLM ({self.model})...")

        try:
            response = completion(
                model=self.model,
                messages=[
                    {"role": "system", "content": self.SYSTEM_PROMPT},
                    {"role": "user", "content": f"URL Fuente: {url_fuente}\n\nTexto del Decreto:\n{texto_decreto[:15000]}"} # Limitamos por contexto
                ],
                api_key=self.api_key,
                response_format={"type": "json_object"} # Forzamos salida JSON si el modelo lo soporta
            )

            raw_content = response.choices[0].message.content
            print("📄 Respuesta LLM recibida.")
            
            # Limpiar posible formato Markdown
            cleaned_content = raw_content.strip()
            if cleaned_content.startswith("```"):
                cleaned_content = re.sub(r"^```(?:json)?\n", "", cleaned_content)
                cleaned_content = re.sub(r"\n```$", "", cleaned_content)
            
            # Parsear y validar con Pydantic
            data = json.loads(cleaned_content)
            
            # El LLM puede devolver una lista de parches si hay varios artículos reformados,
            # pero por ahora el esquema PatchCandidate es individual. 
            # Si el LLM devuelve un dict con 'parches', tomamos el primero como prueba.
            if "parches" in data and isinstance(data["parches"], list):
                data = data["parches"][0]

            # Inyectamos metadatos que el LLM puede no tener precisos
            data["decreto_fuente_url"] = url_fuente
            # La fecha DOF la pondremos como hoy para el prototipo si no viene
            if "fecha_publicacion_dof" not in data:
                 from datetime import date
                 data["fecha_publicacion_dof"] = date.today().isoformat()

            patch = PatchCandidate.model_validate(data)
            return patch

        except Exception as e:
            print(f"❌ Error parseando decreto con LLM: {e}")
            raise LLMParserError(f"Error parseando decreto: {str(e)}")

if __name__ == "__main__":
    # Test (mockeado)
    async def test():
        client = LLMClient()
        # Mock de texto de decreto
        test_text = "DECRETO por el que se reforma el Artículo 1o. del CFF: Las personas... pagarán más."
        # try:
        #     res = await client.parse_decreto_to_patch(test_text, "https://dof.gob.mx/nota_detalle.php?codigo=56789")
        #     print(res.json())
        # except: pass
    
    # asyncio.run(test())
