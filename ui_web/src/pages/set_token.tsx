import { useEffect } from "react";
import { UtokenStore } from "../sutils.ts/utokens";
import type { AuthToken } from "../sutils.ts/utokens";

export const SetToken = () => {
    useEffect(() => {
        const json = new URLSearchParams(window.location.search).get("token");
        const token = JSON.parse(json || "null") as AuthToken | null;
        if (!token) return;
        UtokenStore.update(token)
        window.close();
    }, []);
    return (
        <>
            <p>login success</p>
        </>
    );
};