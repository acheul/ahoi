/**
 * Type-level tests: ret resolution must work converter-agnostically —
 * key types here come from ts-rs output (structural, externally-tagged),
 * ret maps from `#[derive(Rets)]`, and Tsain-style `ret`-branded keys
 * must resolve from their brand. Checked by `pnpm run test` (tsc only).
 */

import type { HailRet, KeyRet, TellRet, VariantOf } from "../src/index.js";
import { createAhoi } from "../src/solid.js";
import type { Hail } from "../../playgrounds/ahoi-wasm-tsrs/bindings/Hail";
import type { Tell } from "../../playgrounds/ahoi-wasm-tsrs/bindings/Tell";
import type { Pier } from "../../playgrounds/ahoi-wasm-tsrs/bindings/Pier";
import type { Fruit } from "../../playgrounds/ahoi-wasm-tsrs/bindings/Fruit";
import type { HailRets, TellRets } from "../../playgrounds/ahoi-wasm-tsrs/bindings/Rets";

type AssertEq<A, B> = A extends B ? (B extends A ? true : never) : never;

// ── variant-name extraction (externally tagged) ─────────────────────────────

const _v1: AssertEq<VariantOf<"Count">, "Count"> = true;
const _v2: AssertEq<VariantOf<{ Item: number }>, "Item"> = true;

// ── ret resolution via the Rets map ─────────────────────────────────────────

const _h1: AssertEq<HailRet<"Count", HailRets>, number> = true;
const _h2: AssertEq<HailRet<{ Item: number }, HailRets>, number | undefined> = true;
const _h3: AssertEq<HailRet<"FruitCounts", HailRets>, Map<string, number>> = true;
const _h4: AssertEq<HailRet<"LastFruit", HailRets>, Fruit | undefined> = true;

const _t1: AssertEq<TellRet<"Increase", TellRets>, number> = true;
const _t2: AssertEq<TellRet<"PopItem", TellRets>, number | undefined> = true;
// un-annotated Tell variants fall back to undefined
const _t3: AssertEq<TellRet<{ PushItem: number }, TellRets>, undefined> = true;
const _t4: AssertEq<TellRet<{ SetFruit: Fruit }, TellRets>, number> = true;

// ── ret resolution via a Tsain-style brand (takes priority over the map) ────

type TsainStyleKey = [0, []] & { readonly __brand: "HailCount"; readonly ret: string[] };
const _b1: AssertEq<KeyRet<TsainStyleKey, HailRets, unknown>, string[]> = true;

// ── solid adapter surface ───────────────────────────────────────────────────

declare const job: import("../src/solid.js").SolidJob<Pier, Hail, Tell>;

const { usePier, PierProvider } = createAhoi<Pier, Hail, Tell, HailRets, TellRets>(job);

const pier = usePier();

// keys are plain wire values — no constructors needed
const count = pier.readHail("Count");
const _c: AssertEq<ReturnType<typeof count>, number> = true;

const [item, setItem] = pier.hail({ Item: 1 });
const _i: AssertEq<ReturnType<typeof item>, number | undefined> = true;
const _is: AssertEq<Parameters<typeof setItem>[0], number | undefined> = true;

const popped = pier.tell("PopItem");
const _p: AssertEq<typeof popped, number | undefined> = true;

const nothing = pier.tell({ PushItem: 5 });
const _n: AssertEq<typeof nothing, undefined> = true;

// cross-key / invalid keys are rejected structurally
// @ts-expect-error - "PopItem" is a Tell variant, not a Hail variant
pier.readHail("PopItem");
// @ts-expect-error - unknown variant
pier.tell("Nope");
// @ts-expect-error - wrong payload type
pier.readHail({ Item: "three" });

// PierProvider accepts only Pier keys
type ProviderProps = Parameters<typeof PierProvider>[0];
const _pp: AssertEq<ProviderProps["pier"], Pier> = true;
const _pier: Pier = "Top";

// suppress unused-variable noise
export const __checked = [
    _v1, _v2, _h1, _h2, _h3, _h4, _t1, _t2, _t3, _t4, _b1,
    _c, _i, _is, _p, _n, _pp, _pier,
] as const;
