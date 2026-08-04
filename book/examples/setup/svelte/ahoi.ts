// #region setup
import wasmInit, {
    abi_version,
    clear,
    hail,
    pier,
    tell,
    write,
} from "../../rust/pkg/ahoi_book_examples";
import { createAhoi } from "@acheul/ahoi-js/svelte";
import type { Pier } from "../../rust/bindings/Pier";
import type { Hail } from "../../rust/bindings/Hail";
import type { Tell } from "../../rust/bindings/Tell";
import type { HailRets, TellRets } from "../../rust/bindings/Rets";

await wasmInit();

// Svelte has no provider component — `providePier` uses `setContext` directly.
export const { providePier, useHail, useReadHail, useTell } = createAhoi<
    Pier,
    Hail,
    Tell,
    HailRets,
    TellRets
>({
    _enrol_pier: pier,
    _enrol_hail: (p, k) => hail(p, k) as [number, any],
    _clear_sphere: clear,
    _write_hail: write,
    _tell: tell,
    _abi_version: abi_version,
});
// #endregion setup
