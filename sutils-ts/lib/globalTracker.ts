export const mouseState = {
    lastMove: performance.now(),
    get IdleTime() { return performance.now() - this.lastMove }
}

document.addEventListener("mousemove", () => {
    mouseState.lastMove = performance.now()
})

document.addEventListener("mouseup", () => {
    mouseState.lastMove = performance.now()
})

export const windowState = {
    changeAt: performance.now(),
    isFocus: document.hasFocus(),
    get focusTime() { return (this.isFocus ? 1 : -1) * (performance.now() - this.changeAt) }
}

window.addEventListener("focus", () => {
    windowState.changeAt = performance.now()
    windowState.isFocus = true
})

window.addEventListener("blur", () => {
    windowState.changeAt = performance.now()
    windowState.isFocus = false
})
