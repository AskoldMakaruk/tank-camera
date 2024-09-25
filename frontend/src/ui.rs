use crate::wasm_bindgen;
use js_sys::Promise;
use log::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::*;

pub fn set_html_label(html_label: &str, session_id: String) {
    let window = web_sys::window().expect("No window Found, We've got bigger problems here");
    let document: Document = window.document().expect("Couldn't Get Document");
    document
        .get_element_by_id(html_label)
        .unwrap_or_else(|| panic!("Should have {} on the page", html_label))
        .dyn_ref::<HtmlLabelElement>()
        .expect("#Button should be a be an `HtmlLabelElement`")
        .set_text_content(Some(&session_id));
}

pub fn get_session_id_from_input() -> String {
    let window = web_sys::window().expect("No window Found, We've got bigger problems here");
    let document: Document = window.document().expect("Couldn't Get Document");
    let sid_input = "sid_input";

    let sid_input = document
        .get_element_by_id(sid_input)
        .unwrap_or_else(|| panic!("Should have {} on the page", sid_input))
        .dyn_ref::<HtmlInputElement>()
        .expect("#HtmlInputElement should be a be an `HtmlInputElement`")
        .value()
        .trim()
        .to_string();
    info!("sid_inputs {}", sid_input);
    sid_input
}

pub fn set_session_connection_status_error(error: String) {
    let window = web_sys::window().expect("No window Found, We've got bigger problems here");
    let document: Document = window.document().expect("Couldn't Get Document");
    let ws_conn_lbl = "session_connection_status_error";

    let e_string;
    if error.is_empty() {
        e_string = format!("")
    } else {
        e_string = format!("Could not connect: {} ", error)
    }

    document
        .get_element_by_id(ws_conn_lbl)
        .unwrap_or_else(|| panic!("Should have {} on the page", ws_conn_lbl))
        .dyn_ref::<HtmlLabelElement>()
        .expect("#Button should be a be an `HtmlLabelElement`")
        .set_text_content(Some(&e_string));
}

#[wasm_bindgen]
pub async fn get_video(video_id: String) -> Result<MediaStream, JsValue> {
    info!("Starting Video Device Capture!");
    let window = web_sys::window().expect("No window Found");
    let navigator = window.navigator();
    let media_devices = match navigator.media_devices() {
        Ok(md) => md,
        Err(e) => return Err(e),
    };

    let constraints = MediaStreamConstraints::new();
    constraints.set_audio(&JsValue::FALSE); // Change this if you want Audio as well !
    constraints.set_video(&JsValue::TRUE);

    let stream_promise: Promise = match media_devices.get_user_media_with_constraints(&constraints)
    {
        Ok(s) => s,
        Err(e) => return Err(e),
    };

    let document: Document = window.document().expect("Couldn't Get Document");

    let video_element: Element = match document.get_element_by_id(&video_id) {
        Some(ms) => ms,
        None => return Err(JsValue::from_str("No Element video found")),
    };

    // debug!("video_element {:?}", video_element);

    let media_stream: MediaStream = match wasm_bindgen_futures::JsFuture::from(stream_promise).await
    {
        Ok(ms) => MediaStream::from(ms),
        Err(e) => {
            error!("{:?}", e);
            error!("{:?}","Its possible that the There is already a tab open with a handle to the Media Stream");
            error!(
                "{:?}",
                "Check if Other tab is open with Video/Audio Stream open"
            );
            return Err(JsValue::from_str("User Did not allow access to the Camera"));
        }
    };

    let vid_elem: HtmlVideoElement = match video_element.dyn_into::<HtmlVideoElement>() {
        Ok(x) => x,
        Err(e) => {
            error!("{:?}", e);
            return Err(JsValue::from_str("User Did not allow access to the Camera"));
        }
    };

    vid_elem.set_src_object(Some(&media_stream));
    Ok(media_stream)
}
