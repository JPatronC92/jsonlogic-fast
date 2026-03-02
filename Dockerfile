FROM python:3.12-slim AS builder

# Install system dependencies needed for Rust and compilation
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    libssl-dev \
    pkg-config \
    libffi-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust via rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

# Upgrade pip and install fundamental build tools
RUN pip install --upgrade pip
RUN pip install uv maturin

# Copy core Rust library
COPY tempus_core /app/tempus_core

# Build the Rust python extension (wheel)
WORKDIR /app/tempus_core
RUN maturin build --release

# -------------------------
# Stage 2: Final Production Image
# -------------------------
FROM python:3.12-slim

WORKDIR /app

# Copy the built Rust wheel from the builder stage
COPY --from=builder /app/tempus_core/target/wheels /tmp/wheels

# Install uv for fast Python package management
RUN pip install --upgrade pip uv

# Copy Python requirements (pyproject.toml/uv.lock)
COPY pyproject.toml uv.lock ./

# Install external dependencies using uv
RUN uv pip install --system -r pyproject.toml

# Install the custom Rust extension specifically built for this architecture
RUN uv pip install --system /tmp/wheels/*.whl

# Copy the actual Python application code
COPY src/ /app/src/
COPY alembic.ini /app/
COPY alembic /app/alembic
COPY scripts/ /app/scripts/

# The API is exposed on Port 8000
EXPOSE 8000

# Start FastAPI via Uvicorn
CMD ["uvicorn", "src.interfaces.api.main:app", "--host", "0.0.0.0", "--port", "8000"]
