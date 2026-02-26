import asyncio
from playwright.async_api import async_playwright
from typing import List, Optional, Dict
from datetime import date
import re

class DOFSpider:
    """
    Scraper real para el Diario Oficial de la Federación (DOF) usando Playwright.
    Extrae decretos de la edición diaria.
    """

    BASE_URL = "https://www.dof.gob.mx/index_detalle.php?fecha="

    async def fetch_decretos_dia(self, query_date: Optional[date] = None) -> List[Dict[str, str]]:
        """
        Busca todos los decretos publicados en una fecha específica.
        Si no se provee fecha, busca la de hoy.
        """
        if query_date is None:
            query_date = date.today()
        
        url = f"{self.BASE_URL}{query_date.strftime('%d/%m/%Y')}"
        print(f"🔍 Buscando decretos en: {url}")

        decretos = []
        async with async_playwright() as p:
            browser = await p.chromium.launch(headless=True)
            page = await browser.new_page()
            
            try:
                await page.goto(url, wait_until="networkidle")
                
                # Buscamos los enlaces que contienen 'nota_detalle.php'
                # El DOF organiza por secciones, usualmente los decretos están en SEGOB o Hacienda.
                links = await page.query_selector_all("a[href*='nota_detalle.php']")
                
                for link in links:
                    text = await link.inner_text()
                    href = await link.get_attribute("href")
                    
                    # Filtramos por palabras clave para no bajar todo el periódico
                    # (Ej: Ley, Código, Decreto, Reglamento, Reforma)
                    if any(kw in text.upper() for kw in ["LEY", "CÓDIGO", "DECRETO", "REGLAMENTO", "REFORMA", "CFA"]):
                        full_url = f"https://www.dof.gob.mx/{href}" if not href.startswith("http") else href
                        decretos.append({
                            "titulo": text.strip(),
                            "url": full_url
                        })
                
                # Eliminamos duplicados
                seen_urls = set()
                decretos = [d for d in decretos if not (d['url'] in seen_urls or seen_urls.add(d['url']))]

            except Exception as e:
                print(f"❌ Error escaneando el DOF: {e}")
            finally:
                await browser.close()
        
        print(f"✅ Se encontraron {len(decretos)} candidatos a decreto.")
        return decretos

    async def fetch_contenido_decreto(self, url: str) -> Optional[str]:
        """
        Extrae el contenido textual (HTML -> Texto) de un decreto específico.
        """
        print(f"📥 Descargando contenido de: {url}")
        
        async with async_playwright() as p:
            browser = await p.chromium.launch(headless=True)
            page = await browser.new_page()
            
            try:
                await page.goto(url, wait_until="domcontentloaded")
                
                # El contenido real de la nota suele estar en un div con id 'nota_detalle'
                # o simplemente extraemos el body filtrando el ruido.
                content_element = await page.query_selector("#nota_detalle")
                if content_element:
                    text = await content_element.inner_text()
                else:
                    # Fallback al body si no encontramos el div específico
                    text = await page.inner_text()
                
                return text.strip()

            except Exception as e:
                print(f"❌ Error bajando contenido de {url}: {e}")
                return None
            finally:
                await browser.close()

if __name__ == "__main__":
    # Test rápido del scraper
    async def test():
        spider = DOFSpider()
        # Buscamos una fecha conocida con actividad (ej: 01/01/2026 o hoy)
        res = await spider.fetch_decretos_dia(date(2025, 2, 24))
        if res:
            print(f"Ejemplo: {res[0]['titulo']}")
            # content = await spider.fetch_contenido_decreto(res[0]['url'])
            # print(content[:500] if content else "Sin contenido")
    
    asyncio.run(test())
