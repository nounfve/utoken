#[derive(Debug, Deref, DerefMut)]
pub struct OptionBearer(Option<Bearer>);

impl<S: Send + Sync> FromRequestParts<S> for OptionBearer {
    type Rejection = TypedHeaderRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        type _H = Option<TypedHeader<Authorization<Bearer>>>;
        let auth = _H::from_request_parts(parts, state)
            .await?
            .map(|auth| auth.0.0);
        OptionBearer(auth).Ok()
    }
}

query_extract!(refresh);
query_extract!(sub);

#[PutInMacro(inline_macro)]
macro_rules! query_extract {
    ($Q:ident) => {
        sutils::external::paste! {
            #[allow(nonstandard_style)]
            #[derive(serde::Deserialize, Debug, Deref, DerefMut)]
            pub struct [<Q_ $Q>](Option<String>);

            impl<S: Send + Sync> FromRequestParts<S> for [<Q_ $Q>] {
                type Rejection = QueryRejection;
                async fn from_request_parts(
                    parts: &mut Parts,
                    state: &S,
                ) -> Result<Self, Self::Rejection> {
                    #[derive(serde::Deserialize, Debug)]
                    struct _Q {
                        $Q: Option<String>,
                    }
                    let query = Query::<_Q>::from_request_parts(parts, state).await?;
                    [<Q_ $Q>](query.0.$Q).Ok()
                }
            }
        }
    };
}

use axum::{
    extract::{FromRequestParts, Query, rejection::QueryRejection},
    http::request::Parts,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
    typed_header::TypedHeaderRejection,
};
use derived_deref::{Deref, DerefMut};
use sutils::{IntoResult, PutInMacro, inline_macro};
