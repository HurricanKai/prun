use crate::data::StarSystem;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

const FIO_API_BASE: &str = "https://rest.fnar.net";

pub async fn fetch_star_systems() -> Result<Vec<StarSystem>, String> {
    let url = format!("{}/systemstars", FIO_API_BASE);
    
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);
    
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
        return Err(format!("HTTP error: {}", resp.status()));
    }
    
    let json = JsFuture::from(resp.json().map_err(|e| format!("JSON error: {:?}", e))?)
        .await
        .map_err(|e| format!("JSON parse error: {:?}", e))?;
    
    let systems: Vec<StarSystem> = serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialization error: {}", e))?;
    
    Ok(systems)
}
