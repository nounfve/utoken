import { ErrorBoundary } from "react-error-boundary";
import { ObjectInLocalStorage } from "../externalStore";
import type { PropsWithChildren } from "react";

export const LastErrorBoundary = ({ children }: PropsWithChildren) => {
    const lastErr = LastError.useAsExternalStore();
    return <ErrorBoundary fallback={
        <div style={{ color: 'red' }}>
            <button onClick={() => LastError.update({ err: undefined })}>⌫</button>
            Error: {lastErr?.msg}
        </div>
    } resetKeys={[lastErr]}>
        <Thorwer></Thorwer>
        {children}
    </ErrorBoundary>
};

const Thorwer = () => {
    const lastErr = LastError.useAsExternalStore();
    if (lastErr?.err) throw lastErr.err
    return <></>
}

const LastError = new ObjectInLocalStorage<{ err: unknown, msg: string }>("last_error");

export const LastErrorCatcher = <F extends (...args: unknown[]) => unknown,>(f: F) => async (...args: unknown[]) => {
    try {
        return await f(...args)
    } catch (err) {
        const msg = JSON.stringify(err);
        LastError.update({ err, msg })
        throw err
    }
}
