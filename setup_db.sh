#!/bin/bash
# setup_db.sh

# 1. Start the database
echo "Starting database..."
docker compose up -d

# 2. Wait for it to be ready
echo "Waiting for database to be ready..."
until docker compose exec db pg_isready -U postgres; do
  echo "Database is unavailable - sleeping"
  sleep 1
done

# 3. Create extension
echo "Creating btree_gist extension..."
docker compose exec db psql -U postgres -d lex_mx_db -c "CREATE EXTENSION IF NOT EXISTS btree_gist;"

echo "Database setup complete."
