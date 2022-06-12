use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex},
};

use mlua::{Error, LuaSerdeExt, UserData, Value};
use reqwest::{
    header::{HeaderMap, HeaderName},
    Method,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct LuaHttp(pub Arc<Mutex<reqwest::Client>>);

#[derive(Serialize, Deserialize, Clone)]
struct LuaHttpRequest {
    url: String,
    method: String,
    body: Option<String>,
    headers: HashMap<String, String>,
}

impl UserData for LuaHttpRequest {}

#[derive(Serialize, Deserialize, Clone)]
struct LuaHttpResponse {
    status: u16,
    body: String,
    headers: Vec<(String, String)>,
}

impl UserData for LuaHttpResponse {}

impl UserData for LuaHttp {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method("request", |l, t, options: Value| async move {
            let options = l.from_value::<LuaHttpRequest>(options)?;

            let client = t.0.lock().unwrap();

            let mut header_map: HeaderMap = HeaderMap::default();
            for (k, v) in options.headers {
                let hn = HeaderName::from_str(&k).unwrap();
                header_map.insert(hn, v.parse().unwrap());
            }

            let result = client
                .request(
                    match options.method.to_uppercase().as_str() {
                        "GET" => Method::GET,
                        "POST" => Method::POST,
                        "PATCH" => Method::PATCH,
                        "OPTIONS" => Method::OPTIONS,
                        "PUT" => Method::PUT,
                        "HEAD" => Method::HEAD,
                        _ => Err(Error::RuntimeError("Invalid METHOD".to_string()))?,
                    },
                    options.url,
                )
                .headers(header_map)
                .body(options.body.unwrap_or_default())
                .send()
                .await
                .or_else(|e| Err(Error::RuntimeError(e.to_string())))?;

            // println!("made req");

            let status = result.status().as_u16();
            let headers = result
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
                .collect::<Vec<_>>();
            let content = result.text().await.unwrap_or("".to_string());

            Ok(l.to_value(&LuaHttpResponse {
                body: content,
                headers,
                status,
            }))
        })
    }

    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(_fields: &mut F) {}
}
