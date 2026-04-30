import { UtokenStore } from "../sutils.ts/utokens"
import { baseCache } from "../sutils.ts/current_page"
import "./account.css"

export const Account = () => {
    const UT = UtokenStore.useAsExternalStore()
    const claim = (<button className="counter">{UT?.claim}</button>)
    const login = (<button className="counter" onClick={() => {
        window.open(`${baseCache}login`, '_blank')?.focus()
    }}>login</button>)
    return (
        <div className="account-page">
            <button>:Account:</button>
            {UT?.access ? claim : login}
        </div>
    )
}
