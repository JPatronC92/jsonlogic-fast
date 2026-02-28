from typing import Generator, Annotated
from fastapi import Depends
from sqlalchemy.ext.asyncio import AsyncSession
from src.infrastructure.database import get_db

SessionDep = Annotated[AsyncSession, Depends(get_db)]
