import requests
import json
from sys import argv
from datetime import date

API_URL = "http://localhost:8000/api/v1"

def print_tree(nodes, level=0):
    for node in nodes:
        indent = "  " * level
        print(f"{indent}├─ [{node['tipo']}] {node['nomenclatura']}")
        # print(f"{indent}│  {node['texto'][:50]}..." if node['texto'] else f"{indent}│  (Sin texto directo)")
        if node['hijos']:
            print_tree(node['hijos'], level + 1)

def main():
    print("
🏛️  LEX API - Cliente de Consola 🏛️
")
    
    # 1. Obtener normas
    print("1. Consultando Leyes disponibles...")
    try:
        res = requests.get(f"{API_URL}/normas")
        res.raise_for_status()
        normas = res.json()
    except requests.exceptions.RequestException as e:
        print(f"❌ Error conectando a la API: {e}. ¿Está corriendo el servidor (uvicorn)?")
        return

    if not normas:
        print("   No hay normas en la base de datos. (Ejecuta el seed_cff_2026.py)")
        return

    for i, n in enumerate(normas):
        print(f"   [{i}] {n['nombre_oficial']} ({n['estado']}) - ID: {n['id']}")

    # 2. Seleccionar la primera norma
    norma_id = normas[0]['id']
    target_date = date.today().isoformat()
    
    if len(argv) > 1:
        target_date = argv[1] # Permitir pasar fecha por CLI
        
    print(f"
2. Descargando estructura para la fecha: {target_date}...")
    
    res = requests.get(f"{API_URL}/normas/{norma_id}/estructura", params={"fecha": target_date})
    if res.status_code == 200:
        arbol = res.json()
        print(f"
📚 Árbol de la Ley ({len(arbol)} nodos principales):
")
        print_tree(arbol)
    else:
        print(f"❌ Error obteniendo estructura: {res.text}")

if __name__ == "__main__":
    main()