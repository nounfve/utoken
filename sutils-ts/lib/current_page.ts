export const currentPage = (path: string = window.location.pathname) => {
    const lastSlash = path.lastIndexOf("/") + 1
    return {
        "host": `${window.location.protocol}//${window.location.host}`,
        "dir": path.substring(0, lastSlash),
        "page": path.substring(lastSlash),
    }
}

export const lastbase = (base: string, path: string = window.location.pathname) => {
    const baseRfind = path.lastIndexOf(base) + base.length;
    const baseUrl = Number.isNaN(baseRfind) ? "/" : path.substring(0, baseRfind);
    cacheBaseParent(baseUrl)
    return baseCache[0]
}

export let baseCache = [""]
export const cacheBaseParent = (base: string) => {
    if (base.at(-1) === "/") {
        base = base.substring(0, base.length - 1)
    }

    baseCache = [base]
    while (true) {
        const idx = base.lastIndexOf("/")
        if (idx < 0) break;
        base = base.substring(0, idx)
        baseCache.push(base)
    }
    console.log(baseCache)
}

export const isDev = () => import.meta.env.DEV
