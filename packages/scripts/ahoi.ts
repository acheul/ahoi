/************************************

  █████╗  ██╗  ██╗  ██████╗  ██╗ ██╗
 ██╔══██╗ ██║  ██║ ██╔═══██╗ ██║ ██║
 ███████║ ███████║ ██║   ██║ ██║ ██║
 ██╔══██║ ██╔══██║ ██║   ██║ ██║ ╚═╝
 ██║  ██║ ██║  ██║ ╚██████╔╝ ██║ ██╗
 ╚═╝  ╚═╝ ╚═╝  ╚═╝  ╚═════╝  ╚═╝ ╚═╝

 ************************************/

export type SphereId = number;

interface Job<PierKey, HailKey> {
    _enrol_pier(p: SphereId | undefined, k: PierKey): SphereId;
    _enrol_hail<T>(p: SphereId, k: HailKey): [SphereId, T];
    _clear_sphere(id: SphereId): void;
    _write_hail<T>(id: SphereId, v: T): void;
}

// Hail Entry
interface HailEntry<T> {
    _key_str: string;
    _count: number;
    _accessor: () => T;
    // write to rust side
    _writer?: (_: T) => void;
    _setter: (_: T) => void;
}

export class AhoiStorage<PierKey, HailKey> {
    /// <id, hail-entry>
    private _hails: Map<
        SphereId,
        HailEntry<any>
    > = new Map();

    private _hail_keys: Map<string, SphereId> = new Map();

    constructor(
        public _job: Job<PierKey, HailKey>,
        public _batch_update: <T>(fn: () => T) => T,
    ) { }

    _enrol_pier = (
        par_pier_id: SphereId | undefined,
        pier_key: PierKey,
        on_clean_up: <F extends () => any>(fn: F) => F,
    ): SphereId => {
        const sphere_id = this._job._enrol_pier(par_pier_id, pier_key);

        on_clean_up(() => {
            this._job._clear_sphere(sphere_id);
        });

        return sphere_id;
    }

    _enrol_hail = <T>(
        par_pier_id: SphereId,
        key: HailKey,
        hat: (_: T) => [() => T, (_: T) => void],
        on_clean_up: <F extends () => any>(fn: F) => F,
    ): [() => T, (_: T) => void] => {
        const [a, b] = this._help_enrol_hail(par_pier_id, key, hat, on_clean_up, true);
        return [a, b!]
    }

    _enrol_read_hail = <T>(
        par_pier_id: SphereId,
        key: HailKey,
        hat: (_: T) => [() => T, (_: T) => void],
        on_clean_up: <F extends () => any>(fn: F) => F,
    ): () => T => {
        const [a, _] = this._help_enrol_hail(par_pier_id, key, hat, on_clean_up, false);
        return a;
    }

    private _help_enrol_hail = <T>(
        par_pier_id: SphereId,
        key: HailKey,
        hat: (_: T) => [() => T, (_: T) => void],
        on_clean_up: <F extends () => any>(fn: F) => F,
        use_write: boolean,
    ): [() => T, ((_: T) => void) | undefined] => {
        let _key_str = JSON.stringify(key);
        let sphere_id = this._hail_keys.get(_key_str);
        let accessor!: (() => T);
        let writer: ((_: T) => void) | undefined;

        if (sphere_id != undefined) {
            // 1) When the hail is already registered:
            //
            let hail = this._hails.get(sphere_id)!;
            // * Increase count
            hail._count++;
            // * use current accessor & writer
            accessor = hail._accessor;
            writer = hail._writer;
        } else {
            // 2) When key is not enrolled:
            //
            let [sphere_id_, value] = this._job._enrol_hail<T>(par_pier_id, key);
            sphere_id = sphere_id_;

            const [_accessor, _setter] = hat(value);
            accessor = _accessor;

            if (use_write) {
                const _writer = (v: T) => {
                    this._job._write_hail<T>(sphere_id_, v);
                };
                writer = _writer;
            }

            // record to _hails
            this._hails.set(sphere_id, {
                _count: 1,
                _key_str,
                _accessor,
                _writer: writer,
                _setter,
            });

            // record to _keys
            this._hail_keys.set(_key_str, sphere_id);
        }

        // add CleanUp Logic
        on_clean_up(() => {
            this._release_hail(sphere_id);
        });

        return [accessor, writer];
    }

    private _release_hail(sphere_id: SphereId): void {
        const sphere = this._hails.get(sphere_id);
        if (!sphere) return;
        sphere._count--;
        if (sphere._count <= 0) {
            // delete from _hails & _keys
            this._hails.delete(sphere_id);
            this._hail_keys.delete(sphere._key_str);
            // cleare sphere
            this._job._clear_sphere(sphere_id);
        }
    }

    _update_hails(hails: (SphereId | any)[]): void {
        this._batch_update(() => {
            for (let i = 0; i < hails.length; i += 2) {
                const sphere_id = hails[i] as SphereId;
                const value = hails[i + 1] as any;
                this._hails.get(sphere_id)?._setter(value);
            }
        });
    }
}

export function buildBridge<PierKey, HailKey>(ahoi: AhoiStorage<PierKey, HailKey>) {
    function hail(hails: (SphereId | any)[]) {
        ahoi._update_hails(hails);
    }
    const BRIDGE = {};
    Object.defineProperty(BRIDGE, "hail", {
        value: hail,
        writable: false,
        configurable: false,
        enumerable: false,
    });
    Object.defineProperty(window, "__AHOI__", {
        value: BRIDGE,
        writable: false,
        configurable: false,
        enumerable: false,
    });
}

