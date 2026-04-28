import type { AuthToken, Token } from "./utokens";

export class LocalToken extends EventTarget implements Partial<AuthToken> {
    access?: Token;
    refresh?: Token;
    claim?: string;
    constructor(jstr: string = "null") {
        super()
        this.from_jstr(jstr)
        this.check_localstorage()
    }

    from_jstr = (jstr: string) => {
        const token = JSON.parse(jstr) as AuthToken | null
        this.access = token?.access
        this.refresh = token?.refresh
        this.claim = token?.claim
    }

    to_authToken = () => ({
        access: this?.access,
        refresh: this?.refresh,
        claim: this?.claim,
    })

    dispatch_utoken = () => {
        const event = new CustomEvent(UtokenStorageKey, {})
        this.dispatchEvent(event)
    }

    listen_utoken = (cb: (u: Partial<AuthToken>) => void) => {
        cb(this.to_authToken())
        this.addEventListener(UtokenStorageKey, ({ target }) => cb((target as LocalToken).to_authToken()))
    }

    storage_listening: boolean = false
    check_localstorage = () => {
        if (this.storage_listening) return;

        this.from_jstr(localStorage.getItem(UtokenStorageKey) || "null")
        this.dispatch_utoken()
        window.addEventListener('storage', event => {
            if (event.key !== UtokenStorageKey) return;
            this.from_jstr(localStorage.getItem(UtokenStorageKey)!)
            this.dispatch_utoken()
        })
    }

    set_localstorage = (token?: AuthToken) => {
        if (token){
            localStorage.setItem(UtokenStorageKey, JSON.stringify(token))
        }else{
            localStorage.removeItem(UtokenStorageKey)
        }
    }
}

const UtokenStorageKey = "utoken"

export const LocalTokenOne = new LocalToken()