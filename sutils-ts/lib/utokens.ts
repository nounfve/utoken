import { ObjectInLocalStorage } from "./externalStore";
export interface Token {
    content: string,
    expire: string,
}

export interface AuthToken {
    claim: string,
    access: Token,
    refresh: Token,
}

export const UtokenStore = new ObjectInLocalStorage<AuthToken>("utoken");

export const utokenAuthInit = (init?: RequestInit) => {
    init = init ?? {}
    init.headers = init.headers ?? {};
    if (UtokenStore.value?.access?.content) {
        (init.headers as Record<string, string>)["Authorization"] = `Bearer ${UtokenStore.value.access.content}`
    }
    return init
}
