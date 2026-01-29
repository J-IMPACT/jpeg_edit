// use web_sys::{js_sys, window};
use yew::prelude::*;

#[function_component(AdSenseAd)]
pub fn adsense_ad() -> Html {
    let ad_client_id = format!("ca-pub-{}", option_env!("AD_CLIENT_ID").unwrap_or("xxxxxxxxxxxxxxxx"));
    let ad_slot = option_env!("AD_SLOT").unwrap_or("xxxxxxxxxx");
    /*
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
    */

    html! {
        <div>
        <p>{format!("Client ID: {}", ad_client_id)}</p>
        <p>{format!("AD slot: {}", ad_slot)}</p>
        /*
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
        */
        </div>
    }
}