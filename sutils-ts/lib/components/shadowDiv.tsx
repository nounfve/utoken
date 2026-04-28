import type { PropsWithChildren } from "react"
import Shadow from 'react-shadow'

export interface ShadowRootPorps {
    cssUrls?: string[]
    Root?: boolean
    Body?: boolean
}

export const ShadowDiv = ({ children, cssUrls, Root, Body }: PropsWithChildren<ShadowRootPorps>) => {
    const styles = cssUrls?.map((url, idx) => (<link rel="stylesheet" href={url} key={idx}></link>))
    let inner = children
    inner = Root ? (<div id='root'>{inner}</div>) : inner
    inner = Body ? (<div id='body'>{inner}</div>) : inner
    return (
        <Shadow.div>
            {styles}
            {inner}
        </Shadow.div>
    )
}