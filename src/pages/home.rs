use yew::prelude::*;
use crate::{albedo_response, albedo};
use wasm_bindgen::JsValue;

pub struct Home
{
    link: ComponentLink<Home>
}

#[derive(Debug)]
pub enum ClientEvent {
    AlbedoRequestLogin,
    AlbedoSuccessLogin(albedo_response::AlbedoPublicKey),
    AlbedoFailLogin(albedo_response::AlbedoError),
    InternalError(serde_json::Error)
}

impl Component for Home {
    type Message = ClientEvent;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self{
            link: link
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            ClientEvent::AlbedoRequestLogin => {
                self.link.send_future(async {
                    let token = "stellar.badge.rs";
                    let albedo_response = albedo::public_key(JsValue::from_str(token)).await;

                    match albedo_response {
                        Ok(resp) => {
                            let res : serde_json::Result<albedo_response::AlbedoPublicKey> = resp.into_serde();
                            match res {
                                Ok(r) => ClientEvent::AlbedoSuccessLogin(r),
                                Err(r) => ClientEvent::InternalError(r)
                            }
                        },
                        Err(resp) => {
                            let res : serde_json::Result<albedo_response::AlbedoError> = resp.into_serde();
                            match res {
                                Ok(r) => ClientEvent::AlbedoFailLogin(r),
                                Err(r) => ClientEvent::InternalError(r)
                            }
                            
                        }
                    }
                });
            },
            ClientEvent::AlbedoSuccessLogin(r) => log::info!("Albedo login successful: {:?}", r),
            ClientEvent::AlbedoFailLogin(r) => log::info!("Albedo login fail: {:?}", r),
            _ => {}
        }

        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html!{
            <button onclick={self.link.callback(|_| ClientEvent::AlbedoRequestLogin)}>{"Albedo"}</button> 
        }
    }

}