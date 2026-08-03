/**
 * SolidJS adapter for the ahoi bridge.
 *
 * Key types come from whatever exporter you use (ts-rs, Tsify, Tsain, ...);
 * the `{Hail,Tell}Rets` maps come from `#[derive(AhoiRets)]` on the Rust side.
 *
 * ```tsx
 * import wasmInit, { pier, hail, clear, write, tell, abi_version } from "../wpkg/wpkg.js";
 * import { createAhoi } from "@acheul/ahoi-js/solid";
 * import type { Pier, Hail, Tell } from "./bindings";      // e.g. ts-rs output
 * import type { HailRets, TellRets } from "./Keys";        // ahoi's TsFile output
 *
 * await wasmInit();
 * export const { PierProvider, usePier } = createAhoi<Pier, Hail, Tell, HailRets, TellRets>({
 *     _enrol_pier: pier,
 *     _enrol_hail: hail,
 *     _clear_sphere: clear,
 *     _write_hail: write,
 *     _tell: tell,
 *     _abi_version: abi_version,
 * });
 *
 * // <PierProvider pier="Top">...</PierProvider>
 * const sphere = usePier();
 * const count = sphere.readHail("Count");          // () => number
 * const item = sphere.readHail({ Item: 3 });       // () => number | undefined
 * const popped = sphere.tell("PopItem");           // boolean
 * ```
 */

import {
    batch,
    createComponent,
    createContext,
    createSignal,
    onCleanup,
    useContext,
    type JSX,
    type ParentProps,
} from "solid-js";
import {
    AhoiStorage,
    type HailRet,
    type Job,
    type SphereId,
    type TellRet,
} from "./index.js";

export interface SolidJob<PierKey, HailKey, TellKey> extends Job<PierKey, HailKey> {
    _tell(sphere_id: SphereId, key: TellKey): any;
}

/** Signal factory handed to AhoiStorage. The extra closure keeps function
 *  values from being mistaken for Solid setter-updaters. */
const hat = <T,>(v: T): [() => T, (nv: T) => void] => {
    const [get, set] = createSignal<T>(v);
    return [get, (nv: T) => set(() => nv)];
};

export interface PierSphere<HailKey, TellKey, HailRets = {}, TellRets = {}> {
    id?: SphereId;
    /** Runs a Tell on this pier's sphere; return type resolved via ret brand or the Rets map. */
    tell<X extends TellKey>(tell: X): TellRet<X, TellRets>;
    /** Subscribes read-only to a Hail; returns a Solid accessor. */
    readHail<X extends HailKey>(key: X): () => HailRet<X, HailRets>;
    /** Subscribes to a Hail with write-back to the Rust side. */
    hail<X extends HailKey>(key: X): [() => HailRet<X, HailRets>, (v: HailRet<X, HailRets>) => void];
}

export interface SolidAhoi<PierKey, HailKey, TellKey, HailRets = {}, TellRets = {}> {
    storage: AhoiStorage<PierKey, HailKey>;
    PierProvider: (props: ParentProps<{ pier: PierKey }>) => JSX.Element;
    /** The nearest PierSphere; throws on use when no provider is above. */
    usePier: () => PierSphere<HailKey, TellKey, HailRets, TellRets>;
    tryUsePier: () => PierSphere<HailKey, TellKey, HailRets, TellRets> | undefined;
}

export function createAhoi<PierKey, HailKey, TellKey, HailRets = {}, TellRets = {}>(
    job: SolidJob<PierKey, HailKey, TellKey>,
): SolidAhoi<PierKey, HailKey, TellKey, HailRets, TellRets> {
    const storage = new AhoiStorage<PierKey, HailKey>(job, batch);

    class Sphere implements PierSphere<HailKey, TellKey, HailRets, TellRets> {
        constructor(public id?: SphereId) { }

        private _id = (): SphereId => {
            if (this.id === undefined) {
                throw new Error("[ahoi] used outside of a <PierProvider>");
            }
            return this.id;
        };

        tell = <X extends TellKey>(tell: X): TellRet<X, TellRets> => {
            return job._tell(this._id(), tell);
        };

        readHail = <X extends HailKey>(key: X): (() => HailRet<X, HailRets>) => {
            return storage._enrol_read_hail<HailRet<X, HailRets>>(this._id(), key, hat, onCleanup);
        };

        hail = <X extends HailKey>(
            key: X,
        ): [() => HailRet<X, HailRets>, (v: HailRet<X, HailRets>) => void] => {
            return storage._enrol_hail<HailRet<X, HailRets>>(this._id(), key, hat, onCleanup);
        };
    }

    const PierContext = createContext<Sphere>();

    const tryUsePier = () => useContext(PierContext);

    const usePier = (): Sphere => tryUsePier() ?? new Sphere();

    const PierProvider = (props: ParentProps<{ pier: PierKey }>): JSX.Element => {
        const par_pier_id = tryUsePier()?.id;
        const pier_id = storage._enrol_pier(par_pier_id, props.pier, onCleanup);
        const sphere = new Sphere(pier_id);
        return createComponent(PierContext.Provider, {
            value: sphere,
            get children() {
                return props.children;
            },
        });
    };

    return { storage, PierProvider, usePier, tryUsePier };
}
