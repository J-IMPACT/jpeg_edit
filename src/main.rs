mod ads;
mod app;
mod exif;
mod map;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
