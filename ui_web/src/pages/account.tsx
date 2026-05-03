import { UtokenStore, type AuthToken } from "../sutils.ts/utokens"
import "./account.css"
import { Vbr } from "../sutils.ts/components/misc"
import { baseCache, currentPage } from "../sutils.ts/current_page"
import { ObjectStore } from "../sutils.ts/externalStore"
import { LastErrorBoundary, LastErrorCatcher } from "../sutils.ts/components/lastError"

export const Account = () => {
    const UT = UtokenStore.useAsExternalStore()
    const [{ menuOpen }, Set] = _State.useAsState();

    const claimOrLogin = UT?.access ? (<button className="counter">{UT?.claim}</button>) : login

    return (
        <div className="account-page">
            <LastErrorBoundary>
                {claimOrLogin}
                <Vbr />
                <div className="account-menu" style={{ display: menuOpen ? "flex" : "none" }}>
                    <button className="counter" onClick={infoToken}>refresh</button>
                    <button className="counter" onClick={clearUtoken}>logout</button>
                    <Vbr />
                </div>
                <button className="counter" disabled={!UT?.access} onClick={() => Set({ menuOpen: !menuOpen })}>{"↩"}</button>
            </LastErrorBoundary>
        </div>
    )
}

const _State = new ObjectStore({ menuOpen: false })

const infoToken = LastErrorCatcher(async () => {
    const access = UtokenStore.value?.access.content;
    const refresh = UtokenStore.value?.refresh.content || "";
    const { host } = currentPage()
    if (!access) return;

    const resp = await fetch(`${host}${baseCache[2]}/token/info?refresh=${refresh}`, {
        headers: { 'Authorization': `Bearer ${access}` }
    })
    if (!resp.ok) throw resp.status;

    const token = await resp.json() as Partial<AuthToken>

    if (token?.access != undefined) {
        UtokenStore.update(token)
    }
    _State.update({ menuOpen: false })
})

const clearUtoken = LastErrorCatcher(async () => {
    const access = UtokenStore.value?.access.content;
    const { host } = currentPage()
    if (!access) return;

    const resp = await fetch(`${host}${baseCache[2]}/token/delete`, {
        method: "DELETE",
        headers: { 'Authorization': `Bearer ${access}` }
    })
    if (!resp.ok) throw resp.status;

    UtokenStore.update(undefined)
    _State.update({ menuOpen: false })
})

const login = <button className="counter" children={"login"} onClick={LastErrorCatcher(() => {
    window.open(`${baseCache[0]}/login`, '_blank')?.focus()
})} />
