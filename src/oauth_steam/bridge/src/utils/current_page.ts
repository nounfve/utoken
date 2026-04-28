export const currentPage = (path: string = window.location.pathname) => {
    const lastSlash = path.lastIndexOf("/") + 1
    return {
        "host": `${window.location.protocol}//${window.location.host}`,
        "dir": path.substring(0, lastSlash),
        "page": path.substring(lastSlash),
    }
}

export let baseCache = "/";
export const lastbase = (base: string, path: string = window.location.pathname) => {
    const baseRfind = path.lastIndexOf(base) + base.length;
    baseCache = path.substring(0, baseRfind);
    return baseCache
}

export const isDev = () => import.meta.env.DEV
