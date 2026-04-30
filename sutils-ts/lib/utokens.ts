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
