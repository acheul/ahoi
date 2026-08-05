/**
 * Wires the wasm module to the solid adapter — Tsain edition.
 *
 * Tsain covers both halves at once: values cross the wire in its positional
 * array format (`TsainConverter` on the Rust side), and the key/data types —
 * with the factory functions and getters that build and read that format —
 * come from the generated `bindings/Tsain.ts`. Ret types ride on the key
 * variants as `ret` brands, so there are no `Rets` maps and no ret-map
 * generics on `createAhoi`.
 */
import wasmInit, {
    abi_version,
    clear,
    hail,
    pier,
    set_panic_hook,
    tell,
    write,
} from "../../ahoi-wasm-tsain-todo/pkg/ahoi_wasm_tsain_todo";
import type { Pier, Hail, Tell } from "../../ahoi-wasm-tsain-todo/bindings/Tsain";
import { createAhoi } from "@acheul/ahoi-js/solid";

// wasm state cannot hot-swap — force a full page reload when this module
// (or the wasm pkg) changes during dev
if (import.meta.hot) import.meta.hot.accept(() => import.meta.hot!.invalidate());

await wasmInit();
set_panic_hook();

export const { PierProvider, usePier } = createAhoi<Pier, Hail, Tell>({
    _enrol_pier: pier,
    _enrol_hail: (p, k) => hail(p, k) as [number, any],
    _clear_sphere: clear,
    _write_hail: write,
    _tell: tell,
    _abi_version: abi_version,
});
