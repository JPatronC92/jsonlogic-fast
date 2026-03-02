# Stage 1: Build the node application
FROM node:20-alpine AS builder

WORKDIR /app

# Install dependencies first for better layer caching
COPY package.json package-lock.json ./
RUN npm ci

# Copy the rest of the Next.js application
COPY . .

# Build the Next.js app 
RUN npm run build

# Stage 2: Production runner
FROM node:20-alpine AS runner

WORKDIR /app

ENV NODE_ENV=production

# Copy necessary files specifically for the Next.js standalone output
# Next.js 12+ supports `output: 'standalone'` in next.config.mjs to reduce target size
COPY --from=builder /app/next.config.ts ./
COPY --from=builder /app/public ./public
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static

EXPOSE 3000

ENV PORT=3000
ENV HOSTNAME="0.0.0.0"

# Note: The standalone output creates a minimal server.js file
CMD ["node", "server.js"]
