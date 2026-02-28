import uuid
from typing import List, Dict, Any, Optional
from datetime import date
from qdrant_client import AsyncQdrantClient
from qdrant_client.http import models
import litellm

from src.core.config import get_settings

settings = get_settings()

class VectorStore:
    def __init__(self, collection_name: str = "lex_mx_knowledge"):
        self.collection_name = collection_name
        self.client = AsyncQdrantClient(host=settings.QDRANT_HOST, port=settings.QDRANT_PORT)
        # Vector size depends on the embedding model
        # text-embedding-3-small -> 1536
        # text-embedding-3-large -> 3072
        # text-embedding-ada-002 -> 1536
        self.vector_size = 1536 

    async def initialize_collection(self):
        """Creates the collection if it doesn't exist."""
        collections = await self.client.get_collections()
        exists = any(c.name == self.collection_name for c in collections.collections)
        
        if not exists:
            await self.client.create_collection(
                collection_name=self.collection_name,
                vectors_config=models.VectorParams(
                    size=self.vector_size,
                    distance=models.Distance.COSINE
                )
            )
            # Create indexes for temporal filtering (Time-Travel)
            await self.client.create_payload_index(
                collection_name=self.collection_name,
                field_name="vigencia_inicio",
                field_schema=models.PayloadSchemaType.DATETIME,
            )
            await self.client.create_payload_index(
                collection_name=self.collection_name,
                field_name="vigencia_fin",
                field_schema=models.PayloadSchemaType.DATETIME,
            )
            print(f"✅ Qdrant Collection '{self.collection_name}' created.")

    async def get_embedding(self, text: str) -> List[float]:
        """Generate embedding for text using LiteLLM."""
        response = await litellm.aembedding(
            model=settings.EMBEDDING_MODEL,
            input=text,
            api_key=settings.LLM_API_KEY
        )
        return response.data[0]["embedding"]

    async def upsert_version(
        self, 
        version_id: uuid.UUID, 
        unidad_id: uuid.UUID,
        texto: str, 
        vigencia_inicio: date, 
        vigencia_fin: Optional[date] = None,
        metadata: Dict[str, Any] = None
    ):
        """Vectorizes a specific version of a legal text with its temporal validity."""
        vector = await self.get_embedding(texto)
        
        payload = {
            "version_id": str(version_id),
            "unidad_id": str(unidad_id),
            "texto": texto,
            # Format dates for Qdrant payload index compatibility (RFC 3339)
            "vigencia_inicio": f"{vigencia_inicio.isoformat()}T00:00:00Z",
            "vigencia_fin": f"{vigencia_fin.isoformat()}T23:59:59Z" if vigencia_fin else None,
        }
        if metadata:
            payload.update(metadata)
            
        point = models.PointStruct(
            id=str(version_id),  # Assuming version_id is unique
            vector=vector,
            payload=payload
        )
        
        await self.client.upsert(
            collection_name=self.collection_name,
            points=[point]
        )

    async def semantic_search(
        self, 
        query: str, 
        fecha_operacion: date, 
        limit: int = 5
    ) -> List[Dict[str, Any]]:
        """
        RAG Legal Definitivo: Semantic search filtered by exact point in time.
        Only returns documents that were legally active on 'fecha_operacion'.
        """
        query_vector = await self.get_embedding(query)
        
        # Format the query date
        query_date_str = f"{fecha_operacion.isoformat()}T12:00:00Z"
        
        # Temporal Filter (Time-Travel): 
        # vigencia_inicio <= fecha_operacion AND (vigencia_fin >= fecha_operacion OR vigencia_fin is NULL)
        temporal_filter = models.Filter(
            must=[
                models.FieldCondition(
                    key="vigencia_inicio",
                    range=models.DatetimeRange(lte=query_date_str)
                ),
                models.Filter(
                    should=[
                        models.FieldCondition(
                            key="vigencia_fin",
                            range=models.DatetimeRange(gte=query_date_str)
                        ),
                        models.IsNullCondition(
                            is_null=models.PayloadField(key="vigencia_fin")
                        )
                    ]
                )
            ]
        )
        
        results = await self.client.search(
            collection_name=self.collection_name,
            query_vector=query_vector,
            query_filter=temporal_filter,
            limit=limit
        )
        
        return [
            {
                "score": result.score,
                "version_id": result.payload["version_id"],
                "texto": result.payload["texto"],
                "vigencia_inicio": result.payload["vigencia_inicio"],
                "vigencia_fin": result.payload.get("vigencia_fin")
            }
            for result in results
        ]
