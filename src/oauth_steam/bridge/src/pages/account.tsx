import { useEffect, useState } from "react"
import type { AuthToken } from "../sutils.ts/utokens"
import { LocalTokenOne } from "../sutils.ts/token_store"
import { baseCache, currentPage } from "../utils/current_page"

export const SteamAccount = () => {
    const [UT, setUT] = useState<Partial<AuthToken>>({})
    useEffect(() => {
        LocalTokenOne.listen_utoken((utoken) => {
            setUT(utoken)
        })
    }, [])
    return (
        <button className="account_button logo_size">
            {UT?.access ? JSON.stringify(UT) : login_button}
        </button>
    )
}

const login_button = (<img
    src="https://steamcdn-a.akamaihd.net/steamcommunity/public/images/steamworks_docs/english/sits_large_border.png"
    alt="sits_large.png"
    title="sits_large.png"
    onClick={() => {
        window.open(steam_login_build(), '_blank')?.focus()
    }}
/>)

const steam_login_build = () => {
    const { host, dir } = currentPage()
    const steam_query = new URLSearchParams({
        'openid.claimed_id': 'http://specs.openid.net/auth/2.0/identifier_select',
        'openid.identity': 'http://specs.openid.net/auth/2.0/identifier_select',
        'openid.mode': 'checkid_setup',
        'openid.ns': 'http://specs.openid.net/auth/2.0',
        'openid.realm': host,
        'openid.return_to': `${host}${baseCache}../verify?on_success=${encodeURIComponent(dir + "callback")}`
    });
    return `https://steamcommunity.com/openid/login?${steam_query}`
}

