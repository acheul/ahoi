/**
 * Vue adapter for the ahoi bridge.
 *
 * `shallowRef` is the signal the storage's `hat` writes into (values arrive
 * from Rust already whole, so there is nothing to track deeply), and
 * `onScopeDispose` releases hails — it fires for components *and* plain
 * `effectScope`s, so composables work outside components too.
 *
 * ```vue
 * <script setup lang="ts">
 * import { useHail, useReadHail, useTell } from "./ahoi";
 *
 * const count = useHail("Count");        // writable ref: count.value++
 * const doubled = useReadHail("Doubled");
 * const tell = useTell();
 * </script>
 *
 * <template>
 *   <p>{{ count }} · {{ doubled }}</p>
 *   <button @click="count++">+1</button>
 *   <button @click="tell('Increase')">tell</button>
 * </template>
 * ```
 *
 * `<PierProvider :pier="..">` enrols its pier during `setup`, so children see
 * the sphere on their first render. The `pier` prop is read once at setup —
 * to switch piers, re-mount the provider (e.g. with a `:key`).
 */

import {
    computed,
    defineComponent,
    inject,
    onScopeDispose,
    provide,
    shallowRef,
    type ComputedRef,
    type InjectionKey,
    type WritableComputedRef,
} from "vue";
import {
    AhoiStorage,
    type HailRet,
    type Job,
    type SphereId,
    type TellRet,
} from "./index.js";

export interface VueJob<PierKey, HailKey, TellKey> extends Job<PierKey, HailKey> {
    _tell(sphere_id: SphereId, key: TellKey): any;
}

/** Component type that keeps `pier` prop checking in templates. */
type PierProviderComponent<PierKey> = new () => { $props: { pier: PierKey } };

export interface VueAhoi<PierKey, HailKey, TellKey, HailRets = {}, TellRets = {}> {
    storage: AhoiStorage<PierKey, HailKey>;
    PierProvider: PierProviderComponent<PierKey>;
    /** The nearest pier's sphere id; throws when no provider is above. */
    usePierId: () => SphereId;
    /** A tell function bound to the nearest pier. */
    useTell: () => <X extends TellKey>(key: X) => TellRet<X, TellRets>;
    /** Subscribes to a hail; the returned ref writes back to the Rust side. */
    useHail: <X extends HailKey>(key: X) => WritableComputedRef<HailRet<X, HailRets>>;
    /** Subscribes read-only to a hail. */
    useReadHail: <X extends HailKey>(key: X) => ComputedRef<HailRet<X, HailRets>>;
}

export function createAhoi<PierKey, HailKey, TellKey, HailRets = {}, TellRets = {}>(
    job: VueJob<PierKey, HailKey, TellKey>,
): VueAhoi<PierKey, HailKey, TellKey, HailRets, TellRets> {
    // Vue's scheduler already coalesces re-renders into a microtask, so the
    // dispatch batch needs no extra wrapper
    const storage = new AhoiStorage<PierKey, HailKey>(job, (fn) => fn());

    const PIER = Symbol("ahoi-pier") as InjectionKey<SphereId>;

    const on_clean_up = <F extends () => any>(fn: F): F => {
        onScopeDispose(fn);
        return fn;
    };

    const hat = <T,>(v: T): [() => T, (nv: T) => void] => {
        const r = shallowRef<T>(v);
        return [() => r.value as T, (nv: T) => { r.value = nv; }];
    };

    const usePierId = (): SphereId => {
        const id = inject(PIER, undefined);
        if (id === undefined) {
            throw new Error("[ahoi] used outside of a <PierProvider>");
        }
        return id;
    };

    const PierProvider = defineComponent({
        name: "PierProvider",
        props: { pier: { required: true } },
        setup(props, { slots }) {
            const par_id = inject(PIER, undefined);
            const id = storage._enrol_pier(par_id, props.pier as PierKey, on_clean_up);
            provide(PIER, id);
            return () => slots.default?.();
        },
    }) as unknown as PierProviderComponent<PierKey>;

    const useTell = () => {
        const pierId = usePierId();
        return <X extends TellKey>(key: X): TellRet<X, TellRets> => job._tell(pierId, key);
    };

    const useHail = <X extends HailKey>(key: X): WritableComputedRef<HailRet<X, HailRets>> => {
        type V = HailRet<X, HailRets>;
        const [read, write] = storage._enrol_hail<V>(usePierId(), key, hat, on_clean_up);
        return computed({ get: read, set: write });
    };

    const useReadHail = <X extends HailKey>(key: X): ComputedRef<HailRet<X, HailRets>> => {
        type V = HailRet<X, HailRets>;
        const read = storage._enrol_read_hail<V>(usePierId(), key, hat, on_clean_up);
        return computed(read);
    };

    return { storage, PierProvider, usePierId, useTell, useHail, useReadHail };
}
