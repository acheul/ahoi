/**
 * React adapter for the ahoi bridge.
 *
 * React has no external signal primitive, so hails are exposed through
 * `useSyncExternalStore` over an adapter-level entry cache:
 *
 * - an entry (one per `pier:key`) holds the current value and its React
 *   subscribers; the storage-level `hat` wires wasm dispatches into it;
 * - entries are created lazily at render (the initial value comes from the
 *   Rust side, so the first render needs the enrolment) and reference-counted
 *   by subscriptions, which only exist in the effect phase;
 * - grace timers reconcile the two phases: an entry created by a render that
 *   never commits (StrictMode discard, suspended tree) is collected after
 *   `ORPHAN_GRACE_MS`, and StrictMode's synchronous unmount→remount cycle is
 *   survived by delaying the release after the last unsubscribe.
 *
 * `<PierProvider>` enrols its pier in an effect (no render side effects) and
 * renders children only once the sphere id exists.
 *
 * ```tsx
 * export const { PierProvider, useHail, useReadHail, useTell } =
 *     createAhoi<Pier, Hail, Tell, HailRets, TellRets>({ ... });
 *
 * // <PierProvider pier="Top">...</PierProvider>
 * const [count, setCount] = useHail("Count");     // [number, (v: number) => void]
 * const doubled = useReadHail("Doubled");         // number
 * const tell = useTell();
 * const popped = tell("PopItem");                 // number | undefined
 * ```
 */

import {
    createContext,
    createElement,
    useCallback,
    useContext,
    useEffect,
    useState,
    useSyncExternalStore,
    type ReactNode,
} from "react";
import {
    AhoiStorage,
    type HailRet,
    type Job,
    type SphereId,
    type TellRet,
} from "./index.js";

export interface ReactJob<PierKey, HailKey, TellKey> extends Job<PierKey, HailKey> {
    _tell(sphere_id: SphereId, key: TellKey): any;
}

interface Entry {
    value: any;
    writer?: (v: any) => void;
    listeners: Set<() => void>;
    refs: number;
    /** releases the storage-side enrolment */
    release: () => void;
}

/** grace before collecting an entry whose render never committed */
const ORPHAN_GRACE_MS = 1000;
/** grace after the last unsubscribe (survives StrictMode's remount cycle) */
const UNSUB_GRACE_MS = 50;

export interface ReactAhoi<PierKey, HailKey, TellKey, HailRets, TellRets> {
    storage: AhoiStorage<PierKey, HailKey>;
    PierProvider: (props: { pier: PierKey; children?: ReactNode }) => ReactNode;
    /** The nearest pier's sphere id; throws when no provider is above. */
    usePierId: () => SphereId;
    /** A tell function bound to the nearest pier. */
    useTell: () => <X extends TellKey>(key: X) => TellRet<X, TellRets>;
    /** Subscribes to a hail with write-back to the Rust side. */
    useHail: <X extends HailKey>(key: X) => [HailRet<X, HailRets>, (v: HailRet<X, HailRets>) => void];
    /** Subscribes read-only to a hail. */
    useReadHail: <X extends HailKey>(key: X) => HailRet<X, HailRets>;
}

export function createAhoi<PierKey, HailKey, TellKey, HailRets = {}, TellRets = {}>(
    job: ReactJob<PierKey, HailKey, TellKey>,
): ReactAhoi<PierKey, HailKey, TellKey, HailRets, TellRets> {
    // dispatches apply synchronously; React 18+ batches the resulting
    // notifications itself, so no batch wrapper is needed
    const storage = new AhoiStorage<PierKey, HailKey>(job, (fn) => fn());

    const entries = new Map<string, Entry>();

    const destroyEntry = (cacheKey: string, entry: Entry) => {
        if (entries.get(cacheKey) === entry) {
            entries.delete(cacheKey);
            entry.release();
        }
    };

    const ensureEntry = (pierId: SphereId, key: HailKey, cacheKey: string): Entry => {
        const existing = entries.get(cacheKey);
        if (existing) return existing;

        const entry: Entry = {
            value: undefined,
            writer: undefined,
            listeners: new Set(),
            refs: 0,
            release: () => { },
        };
        const releases: (() => any)[] = [];
        const on_clean_up = <F extends () => any>(fn: F): F => {
            releases.push(fn);
            return fn;
        };
        // the hat routes wasm dispatches into the entry
        const hat = (v: any): [() => any, (nv: any) => void] => {
            entry.value = v;
            return [
                () => entry.value,
                (nv: any) => {
                    entry.value = nv;
                    entry.listeners.forEach((l) => l());
                },
            ];
        };
        // always enrol writable; the writer is only exposed via `useHail`
        const [, writer] = storage._enrol_hail<any>(pierId, key, hat, on_clean_up);
        entry.writer = writer;
        entry.release = () => releases.forEach((f) => f());
        entries.set(cacheKey, entry);

        setTimeout(() => {
            if (entry.refs === 0) destroyEntry(cacheKey, entry);
        }, ORPHAN_GRACE_MS);

        return entry;
    };

    // ── context / provider ──────────────────────────────────────────────────

    const PierContext = createContext<SphereId | undefined>(undefined);

    function usePierId(): SphereId {
        const id = useContext(PierContext);
        if (id === undefined) {
            throw new Error("[ahoi] used outside of a <PierProvider>");
        }
        return id;
    }

    function PierProvider(props: { pier: PierKey; children?: ReactNode }): ReactNode {
        const par_id = useContext(PierContext);
        const [id, setId] = useState<SphereId>();
        const pier_key_str = JSON.stringify(props.pier);

        useEffect(() => {
            const sphere_id = job._enrol_pier(par_id, props.pier);
            setId(sphere_id);
            return () => {
                setId(undefined);
                job._clear_sphere(sphere_id);
            };
            // eslint-disable-next-line react-hooks/exhaustive-deps
        }, [par_id, pier_key_str]);

        // children may enrol hails, so they must wait for the sphere
        if (id === undefined) return null;
        return createElement(PierContext.Provider, { value: id }, props.children);
    }

    // ── hail hooks ──────────────────────────────────────────────────────────

    function useHailEntry<X extends HailKey>(key: X): { value: any; cacheKey: string } {
        const pierId = usePierId();
        const cacheKey = `${pierId}:${JSON.stringify(key)}`;

        // render-phase creation: the initial value comes from the enrolment;
        // idempotent via the cache, orphan-collected if the render is discarded
        ensureEntry(pierId, key, cacheKey);

        const subscribe = useCallback(
            (listener: () => void) => {
                const entry = ensureEntry(pierId, key, cacheKey); // may re-create after collection
                entry.refs++;
                entry.listeners.add(listener);
                return () => {
                    entry.refs--;
                    entry.listeners.delete(listener);
                    if (entry.refs === 0) {
                        setTimeout(() => {
                            if (entry.refs === 0) destroyEntry(cacheKey, entry);
                        }, UNSUB_GRACE_MS);
                    }
                };
            },
            // eslint-disable-next-line react-hooks/exhaustive-deps
            [cacheKey],
        );
        const getSnapshot = useCallback(
            () => ensureEntry(pierId, key, cacheKey).value,
            // eslint-disable-next-line react-hooks/exhaustive-deps
            [cacheKey],
        );

        const value = useSyncExternalStore(subscribe, getSnapshot);
        return { value, cacheKey };
    }

    function useReadHail<X extends HailKey>(key: X): HailRet<X, HailRets> {
        return useHailEntry(key).value;
    }

    function useHail<X extends HailKey>(
        key: X,
    ): [HailRet<X, HailRets>, (v: HailRet<X, HailRets>) => void] {
        const { value, cacheKey } = useHailEntry(key);
        const write = useCallback(
            (v: any) => entries.get(cacheKey)?.writer?.(v),
            [cacheKey],
        );
        return [value, write];
    }

    // ── tell ────────────────────────────────────────────────────────────────

    function useTell(): <X extends TellKey>(key: X) => TellRet<X, TellRets> {
        const pierId = usePierId();
        return useCallback(
            <X extends TellKey>(key: X): TellRet<X, TellRets> => job._tell(pierId, key),
            [pierId],
        );
    }

    return { storage, PierProvider, usePierId, useTell, useHail, useReadHail };
}
