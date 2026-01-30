use std::path::Path;

use chrono::Local;
use gloo::file::File;
use gloo_dialogs::alert;
use little_exif::{exif_tag::ExifTag, filetype::FileExtension, metadata::Metadata};
use web_sys::{js_sys::{self, Uint8Array}, wasm_bindgen::JsCast, window, Blob, HtmlInputElement, Url};
use yew::prelude::*;

use super::ads::AdSenseAd;
use super::exif::*;
use super::map::MapComponent;

#[function_component(App)]
pub fn app() -> Html {
    let image_url = use_state(|| None::<String>);
    let file_name = use_state(|| None::<String>);
    let latlng = use_state(|| None::<(f64, f64)>);
    let reader = use_state(|| None);
    let metadata = use_state(|| None);
    let file_bytes = use_state(|| None);

    let on_file_change = {
        let image_url = image_url.clone();
        let file_name = file_name.clone();
        let latlng = latlng.clone();
        let reader = reader.clone();
        let metadata = metadata.clone();
        let file_bytes = file_bytes.clone();

        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(file) = input.files().and_then(|f| f.get(0)) {
                if file.type_() != "image/jpeg" {
                    alert("JPEG画像にのみ対応しています");
                    return;
                }

                let file_size = file.size();
                if file_size < 10_240.0 || file_size > 5_242_880.0 {
                    alert("アップロードできるファイルサイズは 10kB ~ 5MB です");
                    return;
                }

                file_name.set(Some(file.name()));
                let url = web_sys::Url::create_object_url_with_blob(&file).unwrap();
                image_url.set(Some(url.clone()));

                let file_reader = gloo::file::callbacks::read_as_bytes(&File::from(file), {
                    let latlng = latlng.clone();
                    let metadata = metadata.clone();
                    let file_bytes = file_bytes.clone();
                    move |result| {
                        if let Ok(bytes) = result {
                            if let Ok(meta) = Metadata::new_from_vec(&bytes, FileExtension::JPEG) {
                                let (mut lat_ref, mut lat, mut lng_ref, mut lng) = (None, None, None, None);
                                for ifd in meta.get_ifds() {
                                    for tag in ifd.get_tags() {
                                        match tag {
                                            ExifTag::GPSLatitudeRef(s) => { lat_ref = dir_to_f64(s); }
                                            ExifTag::GPSLatitude(v) => { lat = dms_to_f64(v); }
                                            ExifTag::GPSLongitudeRef(s) => { lng_ref = dir_to_f64(s); }
                                            ExifTag::GPSLongitude(v) => { lng = dms_to_f64(v); }
                                            _ => {}
                                        }
                                    }
                                }
                                if let (Some(lat_ref), Some(lat), Some(lng_ref), Some(lng)) = (lat_ref, lat, lng_ref, lng) {
                                    latlng.set(Some((lat_ref * lat, lng_ref * lng)));
                                }
                                metadata.set(Some(meta));
                            }
                            file_bytes.set(Some(bytes));
                        }
                    }
                });
                reader.set(Some(file_reader)); // 無いと動かない
            }
        })
    };

    let on_position_change = {
        let latlng = latlng.clone();
        Callback::from(move |(lat, lng): (f64, f64)| {
            latlng.set(Some((lat, lng)));
        })
    };

    let on_download = {
        let file_name = file_name.clone();
        let latlng = latlng.clone();
        let metadata = metadata.clone();
        Callback::from(move |_: MouseEvent| {
            if let (
                Some(name),
                Some((lat, lng)),
                Some(meta), 
                Some(bytes),
            ) = (
                &*file_name,
                *latlng,
                &*metadata, 
                &*file_bytes
            ) {
                let path = Path::new(name);
                let opt_stem = path.file_stem().and_then(|s| s.to_str());
                let opt_extension = path.extension().and_then(|e| e.to_str());

                let (lat_ref, lat, lng_ref, lng) = latlng_to_exif(lat, lng);
                let mut meta = meta.clone();
                let mut bytes = bytes.clone();
                meta.set_tag(ExifTag::GPSLatitudeRef(lat_ref));
                meta.set_tag(ExifTag::GPSLatitude(lat));
                meta.set_tag(ExifTag::GPSLongitudeRef(lng_ref));
                meta.set_tag(ExifTag::GPSLongitude(lng));
                match (meta.write_to_vec(&mut bytes, FileExtension::JPEG), opt_stem, opt_extension) {
                    (Ok(()), Some(stem), Some(extension)) => {
                        let now = Local::now();
                        let formatted = now.format("%H_%M_%S").to_string();
                        let filename = format!("{}_{}.{}", stem, formatted, extension);
                        
                        let uint8_array = Uint8Array::new_with_length(bytes.len() as u32);
                        uint8_array.copy_from(&bytes);
                        let array = js_sys::Array::new();
                        array.push(&uint8_array.buffer());

                        match Blob::new_with_u8_array_sequence(&array) {
                            Ok(blob) => {
                                let url = Url::create_object_url_with_blob(&blob);
                                let window = window().unwrap();
                                let document = window.document().unwrap();
                                match (document.create_element("a"), url) {
                                    (Ok(anchor), Ok(url)) => {
                                        let res_url = anchor.set_attribute("href", &url);
                                        let res_name = anchor.set_attribute("download", &filename);
                                        match (document.body(), res_url, res_name) {
                                            (Some(body), Ok(()), Ok(())) => {
                                                let res_child = body.append_child(&anchor);
                                                let res_ref = anchor.dyn_ref::<web_sys::HtmlElement>();
                                                match (res_child, res_ref) {
                                                    (Ok(_), Some(dr)) => {
                                                        dr.click();
                                                        let _ = Url::revoke_object_url(&url);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        })
    };

    html! {
        <div id="app">
            <div class="form-section">
                <label class="form-label" for="fileInput">{"Upload JPEG Image"}</label>
                <input id="fileInput" type="file" accept="image/jpeg" onchange={on_file_change} />
            </div>

            if let Some(url) = &*image_url {
                <img src={url.clone()} class="preview" />

                if let Some((lat, lng)) = &*latlng {
                    <div class="form-section">
                        <label class="form-label">{"緯度"}</label>
                        <input type="text" value={lat.to_string()} readonly=true />

                        <label class="form-label">{"経度"}</label>
                        <input type="text" value={lng.to_string()} readonly=true />
                    </div>

                    <div class="map-container" id="map">
                        <MapComponent {lat} {lng} {on_position_change} />
                    </div>

                    <div class="button-row">
                        <button class="secondary" onclick={on_download}>{ "ダウンロード" }</button>
                    </div>
                } else {
                    <button class="secondary" onclick={
                        let latlng = latlng.clone();
                        Callback::from(move |_| {
                            latlng.set(Some((36.695055289275, 137.21132191834))); // 富山県庁
                        })
                    }>{ "GPS情報を追加する" }</button>
                }
            }
            <AdSenseAd />
        </div>
    }
}