use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex}, time::Duration,
};

use mlua::{Error, LuaSerdeExt, UserData, Value};
use reqwest::{
    header::{HeaderMap, HeaderName},
    Method,
};
use serde::{Deserialize, Serialize};

use super::permissions::{PERMISSIONS_MANAGER, Permission};

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

            if let Ok(u) = url::Url::parse(&options.url) {
                let domain = u.host_str().ok_or(Error::RuntimeError("invalid url".to_string()))?;
                let p = Permission::Http(domain.to_string());
                PERMISSIONS_MANAGER.lock().unwrap().ask_for_access(&p)?;
            } else {
                return Err(Error::RuntimeError("Invalid URL".to_string()));
            }


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
                .timeout(Duration::from_secs(1)) // 2 mins
                .send()
                .await
                .or_else(|e| Err(Error::ExternalError(Arc::new(e))))?;

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
