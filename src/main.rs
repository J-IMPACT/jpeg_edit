use gloo::file::File;
use gloo_dialogs::alert;
use leaflet::{DragEndEvent, LatLng, Map, MapOptions, Marker, MarkerOptions, TileLayer};
use little_exif::{exif_tag::ExifTag, filetype::FileExtension, metadata::Metadata, rational::uR64};
use web_sys::{js_sys::{self, Uint8Array}, wasm_bindgen::{prelude::Closure, JsCast}, window, Blob, HtmlElement, HtmlInputElement, Url};
use yew::prelude::*;

#[function_component(AdSenseAd)]
pub fn adsense_ad() -> Html {
    let ad_client_id = format!("ca-pub-{}", option_env!("AD_CLIENT_ID").unwrap_or("xxxxxxxxxxxxxxxx"));
    let ad_slot = option_env!("AD_SLOT").unwrap_or("xxxxxxxxxx");
    use_effect(|| {
        if let Some(_window) = window() {
            let _ = js_sys::eval(
                r#"
                (adsbygoogle = window.adsbygoogle || []).push({});
                "#
            );
        }
        || ()
    });


    html! {
        <div>
        <p>{format!("{} {}", ad_client_id, ad_slot)}</p>
        <script async=true src={format!("https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client={}", ad_client_id)} 
            crossorigin="anonymous">
        </script>

        <ins class="adsbygoogle"
            style="display:block"
            data-ad-client={ad_client_id}
            data-ad-slot={ad_slot}
            data-ad-format="auto"
            data-full-width-responsive="true"></ins>
        <script>
            { r#"(adsbygoogle = window.adsbygoogle || []).push({});"# }
        </script>
        </div>
    }
}

fn ur64_to_f64(u: &uR64) -> f64 {
    u.nominator as f64 / u.denominator as f64
}

fn dms_to_f64(v: &[uR64]) -> Option<f64> {
    if v.len() != 3 { return None; }
    Some(ur64_to_f64(&v[0]) + ur64_to_f64(&v[1]) / 60.0 + ur64_to_f64(&v[2]) / 3600.0)
}

fn dir_to_f64(s: &str) -> Option<f64> {
    match s {
        "N" | "E" => Some(1.0),
        "S" | "W" => Some(-1.0),
        _ => None,
    }
}

fn f64_to_dms(value: f64) -> Option<Vec<uR64>> {
    if value < 0.0 { return None; }
    let (mut v, mut value) = (Vec::with_capacity(3), value);
    for i in 0..3 {
        v.push(value as u32);
        value = (value - v[i] as f64) * 60.0;
    }
    Some(vec![
        uR64 { nominator: v[0], denominator: 1 },
        uR64 { nominator: v[1], denominator: 1 },
        uR64 { nominator: v[2] * 100, denominator: 100 }
    ])
}

fn latlng_to_exif(lat: f64, lng: f64) -> (String, Vec<uR64>, String, Vec<uR64>) {
    (
        if lat >= 0.0 { "N".to_string() } else { "S".to_string() },
        f64_to_dms(lat.abs()).unwrap(),
        if lng >= 0.0 { "E".to_string() } else { "W".to_string() },
        f64_to_dms(lng.abs()).unwrap(),
    )
}

#[function_component(App)]
fn app() -> Html {
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
                let (lat_ref, lat, lng_ref, lng) = latlng_to_exif(lat, lng);
                let mut meta = meta.clone();
                let mut bytes = bytes.clone();
                meta.set_tag(ExifTag::GPSLatitudeRef(lat_ref));
                meta.set_tag(ExifTag::GPSLatitude(lat));
                meta.set_tag(ExifTag::GPSLongitudeRef(lng_ref));
                meta.set_tag(ExifTag::GPSLongitude(lng));
                match meta.write_to_vec(&mut bytes, FileExtension::JPEG) {
                    Ok(()) => {
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
                                        let res_name = anchor.set_attribute("download", name);
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
                    Err(_) => {}
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
                            latlng.set(Some((0.0, 0.0)));
                        })
                    }>{ "GPS情報を追加する" }</button>
                }
            }
            <AdSenseAd />
        </div>
    }
}

#[derive(PartialEq, Properties)]
struct MapProps {
    pub lat: f64,
    pub lng: f64,
    pub on_position_change: Callback<(f64, f64)>,
}

struct MapComponent {
    node_ref: NodeRef,
    map: Option<Map>,
    marker: Option<Marker>,
}

impl Component for MapComponent {
    type Message = ();
    type Properties = MapProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { node_ref: NodeRef::default(), map: None, marker: None }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let props = ctx.props();
        let latlng = LatLng::new(props.lat, props.lng);

        if first_render {
            if let Some(container) = self.node_ref.cast::<HtmlElement>() {
                let map = Map::new_with_element(&container, &MapOptions::default());
                map.set_view(&latlng, 11.0);

                TileLayer::new(
                    "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
                ).add_to(&map);
    
                let marker_options = MarkerOptions::new();
                marker_options.set_draggable(true);
                let marker = Marker::new_with_options(&latlng, &marker_options);

                let on_position_change = props.on_position_change.clone();
                let marker_clone = marker.clone();
                let closure = Closure::wrap(Box::new(move |_: DragEndEvent| {
                    let new_pos = marker_clone.get_lat_lng();
                    on_position_change.emit((new_pos.lat(), new_pos.lng()))
                }) as Box<dyn FnMut(DragEndEvent)>);
                marker.on("dragend", closure.as_ref().unchecked_ref());
                closure.forget();
                marker.add_to(&map);
    
                self.map = Some(map);
                self.marker = Some(marker);
            }
        } else {
            if let (Some(map), Some(marker)) = (&self.map, &self.marker) {
                marker.set_lat_lng(&latlng);
                map.set_view(&latlng, map.get_zoom());
            }
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn changed(&mut self, _ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div
                ref={self.node_ref.clone()}
                style="height: 300px; width: 100%;"
                class="map-container"
            />
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
