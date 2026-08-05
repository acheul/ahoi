/**
 * Wires the wasm module to the solid adapter — Tsify edition.
 *
 * The only difference from `playgrounds/solid`: the key types (`Pier`,
 * `Hail`, `Tell`) come from the wasm pkg's own `.d.ts`, where Tsify embedded
 * them, instead of a ts-rs `bindings/` directory. The ret maps still come
 * from ahoi's generated `bindings/Rets.ts`.
 */
import wasmInit, {
    abi_version,
    clear,
    hail,
    pier,
    set_panic_hook,
    tell,
    write,
} from "../../ahoi-wasm-tsify/pkg/ahoi_wasm_tsify";
import type { Pier, Hail, Tell } from "../../ahoi-wasm-tsify/pkg/ahoi_wasm_tsify";
import type { HailRets, TellRets } from "../../ahoi-wasm-tsify/bindings/Rets";
import { createAhoi } from "@acheul/ahoi-js/solid";

// wasm state cannot hot-swap — force a full page reload when this module
// (or the wasm pkg) changes during dev
if (import.meta.hot) import.meta.hot.accept(() => import.meta.hot!.invalidate());

await wasmInit();
set_panic_hook();

export const { PierProvider, usePier } = createAhoi<Pier, Hail, Tell, HailRets, TellRets>({
    _enrol_pier: pier,
    _enrol_hail: (p, k) => hail(p, k) as [number, any],
    _clear_sphere: clear,
    _write_hail: write,
    _tell: tell,
    _abi_version: abi_version,
});
