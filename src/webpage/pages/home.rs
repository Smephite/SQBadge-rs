use crate::js::{albedo, albedo_response};
use crate::webpage::view::Route;
use js_sys::JsString;
use yew::prelude::*;

pub struct Home {
    link: ComponentLink<Home>,
}

#[derive(Debug)]
pub enum ClientEvent {
    AlbedoRequestLogin,
    AlbedoSuccessLogin(albedo_response::AlbedoPublicKey),
    AlbedoFailLogin(albedo_response::AlbedoError),
    InternalError(serde_json::Error),
    Fetch,
}

impl Component for Home {
    type Message = ClientEvent;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link: link }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            ClientEvent::AlbedoRequestLogin => {
                self.link.send_future(async {
                    let token = "stellar.badge.rs";
                    let albedo_response =
                        unsafe { albedo::public_key(JsString::from(token)).await };

                    match albedo_response {
                        Ok(resp) => {
                            let res: serde_json::Result<albedo_response::AlbedoPublicKey> =
                                resp.into_serde();
                            match res {
                                Ok(r) => ClientEvent::AlbedoSuccessLogin(r),
                                Err(r) => ClientEvent::InternalError(r),
                            }
                        }
                        Err(resp) => {
                            let res: serde_json::Result<albedo_response::AlbedoError> =
                                resp.into_serde();
                            match res {
                                Ok(r) => ClientEvent::AlbedoFailLogin(r),
                                Err(r) => ClientEvent::InternalError(r),
                            }
                        }
                    }
                });
            }
            ClientEvent::AlbedoSuccessLogin(r) => {
                yew_router::push_route(Route::Account { id: r.pubkey })
            }
            ClientEvent::AlbedoFailLogin(r) => log::info!("Albedo login fail: {:?}", r),
            ClientEvent::Fetch => {}
            _ => {}
        }

        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="container is-max-desktop">
                   <div class="sqb-main-page">
                        <h1 class="title is-centered">{"SQ Badge Checker"}</h1>
                        <h2 class="subtitle is-centered">{"Only here!"}</h2>
                        <div class="columns is-4 is-variable">
                            <div class="column is-centered">
                                <button class="button is-primary" style="width: 100%"
                                        onclick={self.link.callback(|_| ClientEvent::AlbedoRequestLogin)}>
                                        {"View your Badges!"}
                                </button>
                            </div>
                            <div class="column">
                                <button class="button is-primary" style="width: 100%"
                                        onclick={self.link.callback(|_| ClientEvent::Fetch)}>
                                        {"Verify Proof."}
                                </button>
                            </div>
                        </div>
                   </div>
            </div>
        }
    }
}
