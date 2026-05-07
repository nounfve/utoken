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

    useAsExternalStore = <R = T>(getter: (val: T) => R = () => this.snapshot as unknown as R) =>
        // eslint-disable-next-line react-hooks/rules-of-hooks
        useSyncExternalStore(this.subscribe, () => getter(this.snapshot))
}

export class ObjectStore<T> extends ExternalStore<T, undefined> {
    value: T = {} as T;
    constructor(value: T) {
        super(() => value);
        this.value = value as T;
        this.dataGetter = () => this.value;
        this.notify_all()
    }

    update(value: Partial<T>) {
        this.replace({ ...this.value, ...value })
    }

    mutate(func: (current: T) => Partial<T>) {
        this.update(func(this.value))
    }

    replace(value: T) {
        this.value = value
        this.notify_all()
    }

    useAsState = (): [T, (t: Partial<T>) => void] => [this.useAsExternalStore(), (val: Partial<T>) => this.update(val)]
}

export class ObjectStoreResetable<T> extends ObjectStore<T> {
    initValue: T;
    constructor(value: T) {
        super(value)
        this.initValue = value
    }
    reset() { this.replace(this.initValue) }
}

export class ObjectInLocalStorage<T> extends ObjectStore<T | undefined> {
    storeKey: string
    constructor(storeKey: string) {
        super(undefined as unknown as T)
        this.storeKey = storeKey
        window.addEventListener('storage', this.onStoreChange)
        // mock event on init
        this.mockEvent()
    }

    mockEvent = () => {
        const event = new Event("storage") as Event & { key: string };
        event.key = this.storeKey;
        window.dispatchEvent(event)
    }

    onStoreChange = (event: StorageEvent) => {
        try {
            if (event.key !== this.storeKey) return;
            const str = localStorage.getItem(this.storeKey)
            const obj = JSON.parse(str!)
            if (!obj) throw "empty obj";
            super.replace(obj)
        } catch {
            super.replace(undefined)
        }
    }

    override replace(value: T | undefined) {
        // update pass through localStorage event
        if (!value) {
            localStorage.removeItem(this.storeKey)
        } else {
            const str = JSON.stringify(value)
            localStorage.setItem(this.storeKey, str)
        }
        this.mockEvent()
    }
}