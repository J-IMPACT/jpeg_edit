use gloo::file::File;
use gloo_dialogs::alert;
use gloo_utils::document;
use leaflet::{LatLng, Map, MapOptions, Marker, TileLayer};
use little_exif::{exif_tag::ExifTag, filetype::FileExtension, metadata::Metadata, rational::uR64};
use std::rc::Rc;
use web_sys::{wasm_bindgen::{prelude::Closure, JsCast}, HtmlElement, HtmlImageElement, HtmlInputElement, Url};
use yew::prelude::*;

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

#[derive(Clone, Copy)]
enum InputForm {
    Width, Height
}

#[function_component(App)]
fn app() -> Html {
    let image_url = use_state(|| None::<String>);
    let width = use_state(|| "".to_string());
    let height =  use_state(|| "".to_string());
    let original_wh = use_state(|| None::<(u32, u32)>);
    let fix_ratio = use_state(|| true);
    let latlng = use_state(|| None::<(f64, f64)>);
    let reader = use_state(|| None);

    let on_file_change = {
        let image_url = image_url.clone();
        let width = width.clone();
        let height = height.clone();
        let original_wh = original_wh.clone();
        let latlng = latlng.clone();
        let reader = reader.clone();

        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(file) = input.files().and_then(|f| f.get(0)) {
                if file.type_() != "image/jpeg" {
                    alert("JPEG only");
                    return;
                }

                let file_size = file.size();
                if file_size < 10_240.0 || file_size > 5_242_880.0 {
                    alert("10kB ~ 5MB");
                    return;
                }

                let url = web_sys::Url::create_object_url_with_blob(&file).unwrap();
                image_url.set(Some(url.clone()));

                let image = Rc::new(HtmlImageElement::new().unwrap());
                let image_clone = image.clone();
                let url_clone = url.clone();
                let (width, height) = (width.clone(), height.clone());
                let original_wh = original_wh.clone();
                let closure = Closure::wrap(Box::new(move || {
                    let (w, h) = (image_clone.width(), image_clone.height());
                    width.set(w.to_string());
                    height.set(h.to_string());
                    original_wh.set(Some((w, h)));
                    Url::revoke_object_url(&url_clone).ok();
                }) as Box<dyn Fn()>);

                image.set_onload(Some(closure.as_ref().unchecked_ref()));
                image.set_src(&url);
                closure.forget();

                let file_reader = gloo::file::callbacks::read_as_bytes(&File::from(file), {
                    let latlng = latlng.clone();
                    move |result| {
                        if let Ok(bytes) = result {
                            if let Ok(metadata) = Metadata::new_from_vec(&bytes, FileExtension::JPEG) {
                                let (mut lat_ref, mut lat, mut lng_ref, mut lng) = (None, None, None, None);
                                for ifd in metadata.get_ifds() {
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
                                } else {
                                    latlng.set(Some((2.0, 2.0)));
                                }
                            } else {
                                latlng.set(Some((1.0, 1.0)));
                            }
                        } else {
                            latlng.set(Some((0.0, 0.0)));
                        }
                    }
                });
                reader.set(Some(file_reader)); // 無いと動かない
            }
        })
    };

    let on_input = |form: InputForm| {
        let width = width.clone();
        let height = height.clone();
        let original_wh = original_wh.clone();
        let fix_ratio = fix_ratio.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let (Ok(value), Some((w, h))) = (input.value().parse::<u32>(), *original_wh) {
                let wh_max = std::cmp::max(w, h);
                if value < 1 || value > std::cmp::max(10000, wh_max) { return; }
                match form {
                    InputForm::Width => { width.set(value.to_string()); },
                    InputForm::Height => { height.set(value.to_string()); },
                }

                if *fix_ratio {
                    match form {
                        InputForm::Width => {
                            height.set(((value as f64 * h as f64 / w as f64) as u32).to_string());
                        }
                        InputForm::Height => {
                            width.set(((value as f64 * w as f64 / h as f64) as u32).to_string());
                        }
                    }
                }
            }
        })
    };

    let on_download = {
        let image_url = image_url.clone();
        let width = width.clone();
        let height = height.clone();

        /*Callback::from(move |_| {
            let document = document();
            //let img: HtmlImageElement = document.create
        })*/
    };

    html! {
        <div>
            <input type="file" accept="image/jpeg" onchange={on_file_change} />
            if let Some(url) = &*image_url {
                <img src={url.clone()} style="max-width: 100%; height: auto;" />
                <div>
                    <label>{"Width: "}</label>
                    <input type="text" value={width.to_string()} oninput={on_input(InputForm::Width)} />
                    {" x "}
                    <label>{"Height: "}</label>
                    <input type="text" value={height.to_string()} oninput={on_input(InputForm::Height)} />
                    <input type="checkbox" checked={*fix_ratio} onchange={
                        let fix_ratio = fix_ratio.clone();
                        Callback::from(move |e: Event| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            fix_ratio.set(input.checked());
                        })
                    } />
                    <label>{"Fix ratio"}</label>
                </div>
            }
            if let (Some(_), Some((lat, lng))) = (&*image_url, &*latlng) {
                <p>{ format!("{} {}", lat, lng) }</p>
                <MapComponent {lat} {lng} />
            }
        </div>
    }
}

#[derive(PartialEq, Properties)]
struct MapProps {
    pub lat: f64,
    pub lng: f64,
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
    
                let marker = Marker::new(&latlng);
                marker.add_to(&map);
    
                self.map = Some(map);
                self.marker = Some(marker);
            }
        } else {
            if let (Some(map), Some(marker)) = (&self.map, &self.marker) {
                marker.set_lat_lng(&latlng);
                map.set_view(&latlng, 11.0);
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
