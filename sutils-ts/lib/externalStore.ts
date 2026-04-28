import { useSyncExternalStore } from "react"

export class ExternalStore<T, K> extends Set<() => void> {
    snapshot: T
    dataGetter: () => T
    constructor(getter: () => T) {
        super()
        this.dataGetter = getter
        this.snapshot = this.dataGetter()
    }

    subscribe = (cb: () => void) => {
        this.add(cb)
        return () => this.delete(cb)
    }


    key: K = undefined as K
    notify_all = (p: K = undefined as K) => {
        this.key = p
        this.snapshot = this.dataGetter()
        for (const listener of this) {
            listener()
        }
    }

    useAsExternalStore = () =>
        // eslint-disable-next-line react-hooks/rules-of-hooks
        useSyncExternalStore(this.subscribe, () => this.snapshot)
}

export class ObjectStore<T> extends ExternalStore<T, undefined> {
    value: T = {} as T;
    constructor(value: T) {
        super(() => value);
        this.value = value as T;
        this.dataGetter = () => this.value;
        this.notify_all()
    }

    update = (value: Partial<T>) => {
        this.value = { ...this.value, ...value }
        this.notify_all()
    }

    mutate = (func: (current: T) => Partial<T>) => {
        const value = func(this.value);
        this.update(value)
    }
}