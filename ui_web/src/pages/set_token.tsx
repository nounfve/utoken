import { useEffect } from "react";
import { LocalTokenOne } from "../sutils.ts/token_store";
import type { AuthToken } from "../sutils.ts/utokens";

export const SetToken = () => {
    useEffect(() => {
        const json = new URLSearchParams(window.location.search).get("token");
        const token = JSON.parse(json || "null") as AuthToken | null;
        if (token) {
            LocalTokenOne.set_localstorage(token);
            window.close();
        }
    }, []);
    return (
        <>
            <p>login success</p>
        </>
    );
};