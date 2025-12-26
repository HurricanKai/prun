use crate::data::{AuthResponse, ExchangeStation, Flight, Ship, Site, StarSystem};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response, Headers};

const FIO_API_BASE: &str = "https://rest.fnar.net";

async fn fetch_json<T: serde::de::DeserializeOwned>(url: &str, auth_token: Option<&str>) -> Result<T, String> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);
    
    if let Some(token) = auth_token {
        let headers = Headers::new().map_err(|e| format!("Failed to create headers: {:?}", e))?;
        headers.set("Authorization", token).map_err(|e| format!("Failed to set auth header: {:?}", e))?;
        opts.set_headers(&headers);
    }
    
    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;
    
    let window = web_sys::window().ok_or("No window object")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;
    
    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| "Response is not a Response object")?;
    
    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()));
    }
    
    let json = JsFuture::from(resp.json().map_err(|e| format!("JSON error: {:?}", e))?)
        .await
        .map_err(|e| format!("JSON parse error: {:?}", e))?;
    
    serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialization error: {}", e))
}

pub async fn fetch_star_systems() -> Result<Vec<StarSystem>, String> {
    let url = format!("{}/systemstars", FIO_API_BASE);
    fetch_json(&url, None).await
}

pub async fn fetch_exchange_stations() -> Result<Vec<ExchangeStation>, String> {
    let url = format!("{}/exchange/station", FIO_API_BASE);
    fetch_json(&url, None).await
}

pub async fn login(username: &str, password: &str) -> Result<AuthResponse, String> {
    let url = format!("{}/auth/login", FIO_API_BASE);
    
    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::Cors);
    
    let headers = Headers::new().map_err(|e| format!("Failed to create headers: {:?}", e))?;
    headers.set("Content-Type", "application/json").map_err(|e| format!("Failed to set content type: {:?}", e))?;
    opts.set_headers(&headers);
    
    let body = serde_json::json!({
        "UserName": username,
        "Password": password
    });
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body.to_string()));
    
    let request = Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;
    
    let window = web_sys::window().ok_or("No window object")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;
    
    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| "Response is not a Response object")?;
    
    if !resp.ok() {
        return Err(format!("Login failed: HTTP {}", resp.status()));
    }
    
    let json = JsFuture::from(resp.json().map_err(|e| format!("JSON error: {:?}", e))?)
        .await
        .map_err(|e| format!("JSON parse error: {:?}", e))?;
    
    serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialization error: {}", e))
}

pub async fn fetch_ships(username: &str, auth_token: &str) -> Result<Vec<Ship>, String> {
    let url = format!("{}/ship/ships/{}", FIO_API_BASE, username);
    fetch_json(&url, Some(auth_token)).await
}

pub async fn fetch_sites(username: &str, auth_token: &str) -> Result<Vec<Site>, String> {
    let url = format!("{}/sites/{}", FIO_API_BASE, username);
    fetch_json(&url, Some(auth_token)).await
}

pub async fn fetch_flights(username: &str, auth_token: &str) -> Result<Vec<Flight>, String> {
    let url = format!("{}/ship/flights/{}", FIO_API_BASE, username);
    fetch_json(&url, Some(auth_token)).await
}
