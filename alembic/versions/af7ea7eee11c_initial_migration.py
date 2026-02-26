"""Initial migration

Revision ID: af7ea7eee11c
Revises:
Create Date: 2026-02-25 10:32:24.929249

"""
from typing import Sequence, Union

from alembic import op
import sqlalchemy as sa
from sqlalchemy.dialects import postgresql

# revision identifiers, used by Alembic.
revision: str = 'af7ea7eee11c'
down_revision: Union[str, Sequence[str], None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    # Create enums
    op.execute("DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'estado_norma') THEN CREATE TYPE estado_norma AS ENUM ('VIGENTE', 'ABROGADA', 'DEROGADA'); END IF; END $$;")
    op.execute("DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'tipo_unidad') THEN CREATE TYPE tipo_unidad AS ENUM ('TITULO', 'CAPITULO', 'ARTICULO', 'FRACCION', 'PARRAFO'); END IF; END $$;")

    # Create table normas
    op.create_table(
        'normas',
        sa.Column('id', sa.UUID(), nullable=False),
        sa.Column('nombre_oficial', sa.String(), nullable=False),
        sa.Column('nombre_corto', sa.String(), nullable=True),
        sa.Column('estado', postgresql.ENUM('VIGENTE', 'ABROGADA', 'DEROGADA', name='estado_norma', create_type=False), nullable=False),
        sa.Column('metadata_norma', postgresql.JSONB(astext_type=sa.Text()), nullable=True),
        sa.PrimaryKeyConstraint('id')
    )
    op.create_index(op.f('ix_normas_nombre_oficial'), 'normas', ['nombre_oficial'], unique=False)
    op.create_index(op.f('ix_normas_nombre_corto'), 'normas', ['nombre_corto'], unique=False)

    # Create table unidades_estructurales
    op.create_table(
        'unidades_estructurales',
        sa.Column('uuid', sa.UUID(), nullable=False),
        sa.Column('norma_id', sa.UUID(), nullable=False),
        sa.Column('padre_id', sa.UUID(), nullable=True),
        sa.Column('tipo_unidad', postgresql.ENUM('TITULO', 'CAPITULO', 'ARTICULO', 'FRACCION', 'PARRAFO', name='tipo_unidad', create_type=False), nullable=False),
        sa.Column('orden_indice', sa.Float(), nullable=False),
        sa.ForeignKeyConstraint(['norma_id'], ['normas.id'], ),
        sa.ForeignKeyConstraint(['padre_id'], ['unidades_estructurales.uuid'], ),
        sa.PrimaryKeyConstraint('uuid')
    )

    # Create table versiones_contenido
    op.create_table(
        'versiones_contenido',
        sa.Column('id', sa.UUID(), nullable=False),
        sa.Column('unidad_uuid', sa.UUID(), nullable=False),
        sa.Column('nomenclatura_visible', sa.String(), nullable=False),
        sa.Column('texto_contenido', sa.Text(), nullable=False),
        sa.Column('hash_contenido', sa.String(length=64), nullable=False),
        sa.Column('vigencia', postgresql.DATERANGE(), nullable=False),
        sa.Column('created_at', sa.Date(), server_default=sa.text('now()'), nullable=False),
        sa.ForeignKeyConstraint(['unidad_uuid'], ['unidades_estructurales.uuid'], ),
        sa.PrimaryKeyConstraint('id'),
        postgresql.ExcludeConstraint(('unidad_uuid', '='), ('vigencia', '&&'), name='evitar_solapamiento_temporal')
    )

    # Create table reglas_compliance
    op.create_table(
        'reglas_compliance',
        sa.Column('id', sa.UUID(), nullable=False),
        sa.Column('version_id', sa.UUID(), nullable=False),
        sa.Column('nombre_regla', sa.String(), nullable=False),
        sa.Column('logica_json', postgresql.JSONB(astext_type=sa.Text()), nullable=False),
        sa.Column('tags', postgresql.JSONB(astext_type=sa.Text()), nullable=True),
        sa.ForeignKeyConstraint(['version_id'], ['versiones_contenido.id'], ),
        sa.PrimaryKeyConstraint('id')
    )


def downgrade() -> None:
    op.drop_table('reglas_compliance')
    op.drop_table('versiones_contenido')
    op.drop_table('unidades_estructurales')
    op.drop_index(op.f('ix_normas_nombre_corto'), table_name='normas')
    op.drop_index(op.f('ix_normas_nombre_oficial'), table_name='normas')
    op.drop_table('normas')
    sa.Enum('TITULO', 'CAPITULO', 'ARTICULO', 'FRACCION', 'PARRAFO', name='tipo_unidad').drop(op.get_bind(), checkfirst=True)
    sa.Enum('VIGENTE', 'ABROGADA', 'DEROGADA', name='estado_norma').drop(op.get_bind(), checkfirst=True)
