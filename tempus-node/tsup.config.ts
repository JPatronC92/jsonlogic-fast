import { defineConfig } from "tsup";

export default defineConfig({
  entry: ["src/index.ts"],
  format: ["cjs", "esm"], // CommonJS (Node.js) y ES Modules (Web/Next.js)
  dts: true, // Generar declarations (.d.ts) para TypeScript
  splitting: false,
  sourcemap: true,
  clean: true,
});
