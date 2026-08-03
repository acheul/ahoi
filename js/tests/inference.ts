/**
 * Type-level tests: ret resolution must work converter-agnostically —
 * key types here come from ts-rs output (structural, externally-tagged),
 * ret maps from `#[derive(AhoiRets)]`, and Tsain-style `ret`-branded keys
 * must resolve from their brand. Checked by `pnpm run test` (tsc only).
 */

import type { HailRet, KeyRet, TellRet, VariantOf } from "../src/index.js";
import { createAhoi } from "../src/solid.js";
import type { Hail } from "../../playgrounds/ahoi-wasm/bindings/Hail";
import type { Tell } from "../../playgrounds/ahoi-wasm/bindings/Tell";
import type { Pier } from "../../playgrounds/ahoi-wasm/bindings/Pier";
import type { Fruit } from "../../playgrounds/ahoi-wasm/bindings/Fruit";
import type { HailRets, TellRets } from "../../playgrounds/ahoi-wasm/bindings/Keys";

type AssertEq<A, B> = A extends B ? (B extends A ? true : never) : never;

// ── variant-name extraction (externally tagged) ─────────────────────────────

const _v1: AssertEq<VariantOf<"Count">, "Count"> = true;
const _v2: AssertEq<VariantOf<{ Item: number }>, "Item"> = true;

// ── ret resolution via the Rets map ─────────────────────────────────────────

const _h1: AssertEq<HailRet<"Count", HailRets>, number> = true;
const _h2: AssertEq<HailRet<{ Item: number }, HailRets>, number | undefined> = true;
const _h3: AssertEq<HailRet<"Fruits", HailRets>, [string, Fruit][]> = true;
const _h4: AssertEq<HailRet<{ Fruit: string }, HailRets>, Fruit | undefined> = true;

const _t1: AssertEq<TellRet<"IncreaseCount", TellRets>, number> = true;
const _t2: AssertEq<TellRet<"PopItem", TellRets>, boolean> = true;
// un-annotated Tell variants fall back to undefined
const _t3: AssertEq<TellRet<{ SetCompInfo: string }, TellRets>, undefined> = true;
const _t4: AssertEq<TellRet<{ InsertFruit: [string, Fruit] }, TellRets>, boolean> = true;

// ── ret resolution via a Tsain-style brand (takes priority over the map) ────

type TsainStyleKey = [0, []] & { readonly __brand: "HailCount"; readonly ret: string[] };
const _b1: AssertEq<KeyRet<TsainStyleKey, HailRets, unknown>, string[]> = true;

// ── solid adapter surface ───────────────────────────────────────────────────

declare const job: import("../src/solid.js").SolidJob<Pier, Hail, Tell>;

const { usePier, PierProvider } = createAhoi<Pier, Hail, Tell, HailRets, TellRets>(job);

const sphere = usePier();

// keys are plain wire values — no constructors needed
const count = sphere.readHail("Count");
const _c: AssertEq<ReturnType<typeof count>, number> = true;

const [item, setItem] = sphere.hail({ Item: 3 });
const _i: AssertEq<ReturnType<typeof item>, number | undefined> = true;
const _is: AssertEq<Parameters<typeof setItem>[0], number | undefined> = true;

const popped = sphere.tell("PopItem");
const _p: AssertEq<typeof popped, boolean> = true;

const nothing = sphere.tell({ SetCompInfo: "hi" });
const _n: AssertEq<typeof nothing, undefined> = true;

// cross-key / invalid keys are still rejected structurally
// @ts-expect-error - "PopItem" is a Tell variant, not a Hail variant
sphere.readHail("PopItem");
// @ts-expect-error - unknown variant
sphere.tell("Nope");
// @ts-expect-error - wrong payload type
sphere.readHail({ Item: "three" });

// PierProvider accepts only Pier keys
type ProviderProps = Parameters<typeof PierProvider>[0];
const _pp: AssertEq<ProviderProps["pier"], Pier> = true;
const _pier: Pier = "Top";

// suppress unused-variable noise
export const __checked = [
    _v1, _v2, _h1, _h2, _h3, _h4, _t1, _t2, _t3, _t4, _b1,
    _c, _i, _is, _p, _n, _pp, _pier,
] as const;
