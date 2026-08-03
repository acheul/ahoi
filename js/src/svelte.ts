/**
 * Svelte adapter for the ahoi bridge (Svelte 4 and 5).
 *
 * Hails are exposed as **stores**, not runes: `$count` auto-subscription works
 * in both versions (runes mode included), and it keeps this adapter plain
 * TypeScript — runes would force the library itself into `.svelte.ts` files
 * and a Svelte compilation step.
 *
 * Piers use Svelte's function-style context instead of a provider component
 * (which would likewise require shipping a compiled `.svelte` file): call
 * `providePier` at the top of the component that owns the pier.
 *
 * ```svelte
 * <script lang="ts">
 *   import { providePier, useHail, useReadHail, useTell } from "./ahoi";
 *
 *   providePier("Top");                    // this component and its children
 *   const count = useHail("Count");        // writable store
 *   const doubled = useReadHail("Doubled");
 *   const tell = useTell();
 * </script>
 *
 * <p>{$count} · {$doubled}</p>
 * <button onclick={() => $count++}>+1</button>
 * <button onclick={() => tell("Increase")}>tell</button>
 * ```
 *
 * All of these must be called during component initialisation (they use
 * `setContext` / `getContext` / `onDestroy`).
 */

import { getContext, onDestroy, setContext } from "svelte";
import { writable, type Readable, type Writable } from "svelte/store";
import {
    AhoiStorage,
    type HailRet,
    type Job,
    type SphereId,
    type TellRet,
} from "./index.js";

export interface SvelteJob<PierKey, HailKey, TellKey> extends Job<PierKey, HailKey> {
    _tell(sphere_id: SphereId, key: TellKey): any;
}

/**
 * The storage hands back the *same* accessor for a key that is already
 * enrolled, so the store that fans a value out to many components rides along
 * on the accessor itself — no adapter-side cache or refcount needed.
 */
type StoreCarrier<T> = (() => T) & { _store: Writable<T> };

export interface SvelteAhoi<PierKey, HailKey, TellKey, HailRets = {}, TellRets = {}> {
    storage: AhoiStorage<PierKey, HailKey>;
    /** Enrols a pier and shares it with this component's subtree. */
    providePier: (pier: PierKey) => SphereId;
    /** The nearest pier's sphere id; throws when no pier was provided above. */
    usePierId: () => SphereId;
    /** A tell function bound to the nearest pier. */
    useTell: () => <X extends TellKey>(key: X) => TellRet<X, TellRets>;
    /** Subscribes to a hail; `$store = v` writes back to the Rust side. */
    useHail: <X extends HailKey>(key: X) => Writable<HailRet<X, HailRets>>;
    /** Subscribes read-only to a hail. */
    useReadHail: <X extends HailKey>(key: X) => Readable<HailRet<X, HailRets>>;
}

export function createAhoi<PierKey, HailKey, TellKey, HailRets = {}, TellRets = {}>(
    job: SvelteJob<PierKey, HailKey, TellKey>,
): SvelteAhoi<PierKey, HailKey, TellKey, HailRets, TellRets> {
    // store subscribers run synchronously; Svelte coalesces the resulting DOM
    // updates itself, so the dispatch batch needs no extra wrapper
    const storage = new AhoiStorage<PierKey, HailKey>(job, (fn) => fn());

    const PIER = Symbol("ahoi-pier");

    const on_clean_up = <F extends () => any>(fn: F): F => {
        onDestroy(fn);
        return fn;
    };

    const hat = <T,>(v: T): [() => T, (nv: T) => void] => {
        let current = v;
        const store = writable<T>(v);
        const read = (() => current) as StoreCarrier<T>;
        read._store = store;
        return [read, (nv: T) => { current = nv; store.set(nv); }];
    };

    const usePierId = (): SphereId => {
        const id = getContext<SphereId | undefined>(PIER);
        if (id === undefined) {
            throw new Error("[ahoi] used without a pier — call providePier() first");
        }
        return id;
    };

    const providePier = (pier: PierKey): SphereId => {
        const par_id = getContext<SphereId | undefined>(PIER);
        const id = storage._enrol_pier(par_id, pier, on_clean_up);
        setContext(PIER, id);
        return id;
    };

    const useTell = () => {
        const pierId = usePierId();
        return <X extends TellKey>(key: X): TellRet<X, TellRets> => job._tell(pierId, key);
    };

    const useHail = <X extends HailKey>(key: X): Writable<HailRet<X, HailRets>> => {
        type V = HailRet<X, HailRets>;
        const [read, write] = storage._enrol_hail<V>(usePierId(), key, hat, on_clean_up);
        const { subscribe } = (read as StoreCarrier<V>)._store;
        return {
            subscribe,
            set: write,
            update: (fn) => write(fn(read())),
        };
    };

    const useReadHail = <X extends HailKey>(key: X): Readable<HailRet<X, HailRets>> => {
        type V = HailRet<X, HailRets>;
        const read = storage._enrol_read_hail<V>(usePierId(), key, hat, on_clean_up);
        return { subscribe: (read as StoreCarrier<V>)._store.subscribe };
    };

    return { storage, providePier, usePierId, useTell, useHail, useReadHail };
}
