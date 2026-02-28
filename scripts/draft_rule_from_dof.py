import asyncio
import json
from src.infrastructure.llm_client import LLMClient

async def run_draft():
    print("🚀 Iniciando Propositor de Reglas Universales...")
    llm = LLMClient()

    esquema_simulado = {
        "type": "object",
        "properties": {
            "monto_compra": {"type": "number"},
            "metodo_pago": {"type": "string"},
            "concepto": {"type": "string"}
        },
        "required": ["monto_compra", "metodo_pago"]
    }

    texto_ley = """
    Artículo 27. Las deducciones autorizadas en este Título deberán reunir los siguientes requisitos:
    ...
    Tratándose de la adquisición de combustibles para vehículos marítimos, aéreos y terrestres,
    el pago deberá efectuarse en forma indefectiblemente mediante transferencia electrónica de fondos,
    tarjeta de crédito, débito o de servicios, o monederos electrónicos autorizados por el SAT,
    aun cuando la contraprestación no exceda de $2,000.00.
    """

    print("\n📜 Ley a procesar:\n", texto_ley.strip())
    print("\n⚙️  Generando lógica json-logic...")

    resultado = await llm.draft_universal_rule(texto_ley, esquema_simulado)

    print("\n✅ Propuesta Generada:")
    print(json.dumps(resultado, indent=2, ensure_ascii=False))

if __name__ == "__main__":
    asyncio.run(run_draft())
