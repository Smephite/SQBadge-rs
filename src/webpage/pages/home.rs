use crate::js::{albedo, albedo_response};
use crate::stellar::*;
use crate::util::badge_check;
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
    None,
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
                    let albedo_response = albedo::public_key(JsString::from(token)).await;

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
                self.link.send_future(async {
                    let pub_key = r.pubkey;
                    let badges = stellar::fetch_toml_currencies(&String::from(
                        "https://quest.stellar.org/.well-known/stellar.toml",
                    ))
                    .await
                    .unwrap();

                    let badges = badges
                        .into_iter()
                        .filter(|b| b.code.starts_with("SQ"))
                        .collect::<Vec<stellar_data::TOMLCurrency>>();

                    let in_possession = badge_check::fetch_badges(&pub_key, &badges).await;
                    log::info!("{}: {:?}", pub_key, in_possession);
                    ClientEvent::None
                });
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
            <>
                <button onclick={self.link.callback(|_| ClientEvent::AlbedoRequestLogin)}>{"Albedo"}</button>
                <button onclick={self.link.callback(|_| ClientEvent::Fetch)}>{"Fetch"}</button>
            </>
        }
    }
}
