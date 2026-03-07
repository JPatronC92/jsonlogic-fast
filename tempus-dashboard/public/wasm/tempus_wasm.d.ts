/* tslint:disable */
/* eslint-disable */

export function evaluate_batch_multi_wasm(rules_json: string, contexts_json: string): string;

export function evaluate_batch_wasm(rule_json: string, contexts_json: string): string;

export function evaluate_fee_wasm(rule_json: string, context_json: string): number;

export function get_core_info_wasm(): string;

export function validate_rule_wasm(rule_json: string): boolean;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly evaluate_batch_multi_wasm: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly evaluate_batch_wasm: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly evaluate_fee_wasm: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly get_core_info_wasm: () => [number, number];
    readonly validate_rule_wasm: (a: number, b: number) => [number, number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
