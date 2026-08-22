use crate::layout::engine::CachedImage;
#[cfg(target_arch = "wasm32")]
use js_sys::{
    ArrayBuffer,
    Uint8Array,
};
#[cfg(all(not(target_arch = "wasm32"), feature = "fetch-async"))]
use reqwest::Client;
use std::sync::Arc;
#[cfg(all(not(target_arch = "wasm32"), feature = "fetch-async"))]
use std::sync::LazyLock;
use usvg::ImageKind;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::{
    Request,
    RequestInit,
    Response,
};

/// Reqwest client
#[cfg(all(not(target_arch = "wasm32"), feature = "fetch-async"))]
static CLIENT: LazyLock<Client> = LazyLock::new(|| Client::new());

/// Fetches an image from a URL asynchronously (for WASM).
#[cfg(target_arch = "wasm32")]
pub(crate) async fn fetch_image_async(href: &str) -> Option<CachedImage> {
    let window = web_sys::window()?;

    let opts = RequestInit::new();
    opts.set_method("GET");
    let req = Request::new_with_str_and_init(href, &opts).ok()?;

    let res_value = JsFuture::from(window.fetch_with_request(&req)).await.ok()?;
    let res: Response = res_value.dyn_into().ok()?;

    if !res.ok() {
        return None;
    }

    let buf: ArrayBuffer = JsFuture::from(res.array_buffer().ok()?)
        .await
        .ok()?
        .dyn_into()
        .ok()?;

    to_cached_image(Arc::new(Uint8Array::new(&buf).to_vec()))
}

/// Fetches an image from a URL asynchronously (for bare-metal).
#[cfg(all(not(target_arch = "wasm32"), feature = "fetch-async"))]
pub(crate) async fn fetch_image_async(href: &str) -> Option<CachedImage> {
    let res = CLIENT.get(href).send().await.ok()?;

    if !res.status().is_success() {
        return None;
    }

    let buf = res.bytes().await.ok()?;
    to_cached_image(Arc::new(buf.to_vec()))
}

/// Fetches an image from a URL synchronously.
#[cfg(all(not(target_arch = "wasm32"), feature = "fetch"))]
pub(crate) fn fetch_image_sync(href: &str) -> Option<CachedImage> {
    let mut res = ureq::get(href).call().ok()?;

    if !res.status().is_success() {
        return None;
    }

    let buf = res.body_mut().read_to_vec().ok()?;
    to_cached_image(Arc::new(buf))
}

/// Converts raw image data into [`CachedImage`].
fn to_cached_image(data: Arc<Vec<u8>>) -> Option<CachedImage> {
    if let Some(kind) = infer::get(&data) {
        match kind.mime_type() {
            "image/png" => return Some(CachedImage::Resolved(Box::new(ImageKind::PNG(data)))),
            "image/jpeg" => return Some(CachedImage::Resolved(Box::new(ImageKind::JPEG(data)))),
            "image/webp" => return Some(CachedImage::Resolved(Box::new(ImageKind::WEBP(data)))),
            "image/gif" => return Some(CachedImage::Resolved(Box::new(ImageKind::GIF(data)))),
            _ => {}
        }
    }

    let to_sniff = data.len().min(512); // sniff over svg design tools meta comments
    let scratch = String::from_utf8_lossy(&data[..to_sniff]).to_lowercase();

    if scratch.contains("<svg") {
        return Some(CachedImage::DeferredSvg(data));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_svg() {
        let data = Arc::new(br#"<svg viewBox="0 0 24 24"></svg>"#.to_vec());
        let result = to_cached_image(data).unwrap();
        assert!(matches!(result, CachedImage::DeferredSvg(_)));
    }

    #[test]
    fn parses_svg_with_leading_commnents() {
        let data = Arc::new(
            br#"<!-- exported using some tool --><svg viewBox="0 0 24 24"></svg>"#.to_vec(),
        );
        let result = to_cached_image(data).unwrap();
        assert!(matches!(result, CachedImage::DeferredSvg(_)));
    }

    #[test]
    fn fails_on_invalid_data() {
        let data = Arc::new(vec![0x00, 0x01, 0x02, 0x03]);
        let result = to_cached_image(data);
        assert!(result.is_none());
    }
}
