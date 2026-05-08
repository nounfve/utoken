import { UtokenStore, type AuthToken } from "../sutils.ts/utokens"
import "./account.css"
import { Vbr } from "../sutils.ts/components/misc"
import { baseCache, currentPage } from "../sutils.ts/current_page"
import { ObjectStore } from "../sutils.ts/externalStore"
import { LastErrorBoundary, LastErrorCatcher } from "../sutils.ts/components/lastError"
import { useEffect, useRef } from "react"
import { windowState } from "../sutils.ts/globalTracker"
import { useSearchParams } from "react-router"

export const Account = () => {
    const UT = UtokenStore.useAsExternalStore()
    const { menuOpen } = _State.useAsExternalStore();
    const [query] = useSearchParams()
    const divRef = useRef<HTMLDivElement>(null)

    const side = query.getAll("side").map(val => `side-${val}`).join(" ")

    const claimOrLogin = UT?.access ? (<button className="counter">{UT?.claim}</button>) : login
    useEffect(() => {
        if (windowState.isTopWindow) return;
        document.documentElement.classList.add("iframe-doc")
        divRef.current!.classList.add("iframe-page")
        divRef.current!.parentElement?.removeAttribute("id")
        divRef.current!.addEventListener("mouseenter", TriggerMenu);
        divRef.current!.addEventListener("mouseleave", () => _State.update({ menuOpen: false }));
    }, [])

    return (
        <div className={`account-page ${side}`} ref={divRef}>
            <LastErrorBoundary>
                {claimOrLogin}
                <div className="account-menu" style={{ display: menuOpen ? "flex" : "none" }}>
                    <Vbr />
                    <button className="counter" onClick={infoToken}>refresh</button>
                    <button className="counter" onClick={clearUtoken}>logout</button>
                </div>
                {windowState.isTopWindow && <>
                    <Vbr />
                    <button className="counter" disabled={!UT?.access} onClick={TriggerMenu}>{"↩"}</button>
                </>}
            </LastErrorBoundary>
        </div>
    )
}

const _State = new ObjectStore({ menuOpen: false })
const TriggerMenu = () => {
    const menuOpen = !_State.value.menuOpen && !!UtokenStore.value?.access
    _State.update({ menuOpen })
}

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
        UtokenStore.replace(token as AuthToken)
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

    UtokenStore.replace(undefined)
    _State.update({ menuOpen: false })
})

const login = <button className="counter" children={"login"} onClick={LastErrorCatcher(() => {
    window.open(`${baseCache[0]}/login`, '_blank')?.focus()
})} />
