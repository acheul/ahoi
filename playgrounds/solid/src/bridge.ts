/**
 * Wires the wasm module to the solid adapter. This is the only
 * framework-specific setup an app needs.
 */
import wasmInit, {
    abi_version,
    clear,
    hail,
    pier,
    set_panic_hook,
    tell,
    write,
} from "../../ahoi-wasm-tsrs/pkg/ahoi_wasm_tsrs";
import { createAhoi } from "@acheul/ahoi-js/solid";
import type { Pier } from "../../ahoi-wasm-tsrs/bindings/Pier";
import type { Hail } from "../../ahoi-wasm-tsrs/bindings/Hail";
import type { Tell } from "../../ahoi-wasm-tsrs/bindings/Tell";
import type { HailRets, TellRets } from "../../ahoi-wasm-tsrs/bindings/Rets";

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
