import { baseCache, currentPage } from "../sutils.ts/current_page"
import "./login.css"

export const Login = () => {
    return (
        <div className="login-page">
            <button>{steam_login}</button>
        </div>
    )
}

const steam_login = (<img
    src="https://steamcdn-a.akamaihd.net/steamcommunity/public/images/steamworks_docs/english/sits_large_border.png"
    alt="sits_large.png"
    title="sits_large.png"
    onClick={() => { window.location.replace(steam_login_build()) }}
/>)

const steam_login_build = () => {
    const { host, dir } = currentPage()
    const steam_query = new URLSearchParams({
        'openid.claimed_id': 'http://specs.openid.net/auth/2.0/identifier_select',
        'openid.identity': 'http://specs.openid.net/auth/2.0/identifier_select',
        'openid.mode': 'checkid_setup',
        'openid.ns': 'http://specs.openid.net/auth/2.0',
        'openid.realm': host,
        'openid.return_to': `${host}${baseCache}../steam/verify?on_success=${encodeURIComponent(dir + ".set_token")}`
    });
    return `https://steamcommunity.com/openid/login?${steam_query}`
}
