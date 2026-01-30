use leaflet::{DragEndEvent, LatLng, Map, MapOptions, Marker, MarkerOptions, TileLayer};
use web_sys::{wasm_bindgen::{prelude::Closure, JsCast}, HtmlElement};
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct MapProps {
    pub lat: f64,
    pub lng: f64,
    pub on_position_change: Callback<(f64, f64)>,
}

pub struct MapComponent {
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