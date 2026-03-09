// WASM Loader — Initializes the Tempus Rust core in the browser
import type { InitOutput } from "../../public/wasm/tempus_wasm";

let wasmModule: InitOutput | null = null;
let wasmFunctions: typeof import("../../public/wasm/tempus_wasm") | null = null;

export async function initWasm() {
    if (wasmModule) return wasmFunctions!;

    const wasm = await import("../../public/wasm/tempus_wasm");
    wasmModule = await wasm.default();
    wasmFunctions = wasm;
    return wasm;
}

export function getWasm() {
    if (!wasmFunctions) throw new Error("WASM not initialized. Call initWasm() first.");
    return wasmFunctions;
}
