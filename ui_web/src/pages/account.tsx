import { ShadowDiv } from "../sutils-ts/components/shadowDiv"

export const Account = () => {
    const isInIframe = window.top !== window.self
    const str = JSON.stringify(isInIframe)
    return (
        <div>
            <p>shadow root</p>
            <button className="counter">iniframe:{str}</button>
            <ShadowDiv>
                <p>shadow root</p>
                <button className="counter">iniframe:{str}</button>
            </ShadowDiv>
        </div>
    )
}