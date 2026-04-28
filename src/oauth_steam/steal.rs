/// steal from https://docs.rs/steam-openid/0.2.0/src/steam_openid/lib.rs.html#100
#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct VerifyForm {
    #[serde(rename = "openid.ns")]
    pub ns: String,
    #[serde(rename = "openid.mode")]
    pub mode: String,
    #[serde(rename = "openid.op_endpoint")]
    pub op_endpoint: String,
    #[serde(rename = "openid.claimed_id")]
    pub claimed_id: String,
    #[serde(rename = "openid.identity")]
    pub identity: Option<String>,
    #[serde(rename = "openid.return_to")]
    pub return_to: String,
    #[serde(rename = "openid.response_nonce")]
    pub response_nonce: String,
    #[serde(rename = "openid.invalidate_handle")]
    pub invalidate_handle: Option<String>,
    #[serde(rename = "openid.assoc_handle")]
    pub assoc_handle: String,
    #[serde(rename = "openid.signed")]
    pub signed: String,
    #[serde(rename = "openid.sig")]
    pub sig: String,
}