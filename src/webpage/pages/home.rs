use crate::js::{albedo, albedo_response};
use crate::webpage::view::Route;
use js_sys::JsString;
use yew::prelude::*;

pub struct Home {
    link: ComponentLink<Home>,
    modal_open: bool,
}

#[derive(Debug)]
pub enum ClientEvent {
    AlbedoRequestLogin,
    AlbedoSuccessLogin(albedo_response::AlbedoPublicKey),
    AlbedoFailLogin(albedo_response::AlbedoError),
    InternalError(serde_json::Error),
    ToggleProofChoice,
    ProofUpload(),
}

impl Component for Home {
    type Message = ClientEvent;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link: link,
            modal_open: false,
        }
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
            ClientEvent::ToggleProofChoice => {
                self.modal_open = !self.modal_open;
                return true;
            }
            ClientEvent::ProofUpload() => {}
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
                <div class="container is-max-desktop">
                    <div class="sqb-centered">
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
                                            onclick={self.link.callback(|_| ClientEvent::ToggleProofChoice)}>
                                            {"Verify Proof"}
                                    </button>
                                </div>
                            </div>
                    </div>
                </div>
                {
                    if self.modal_open {
                        self.render_modal()
                    } else {
                        Html::default()
                    }
                }
            </>
        }
    }
}

impl Home {
    fn render_modal(&self) -> Html {
        html! {
            <div class="modal is-active">
                <div class="modal-background" onclick={self.link.callback(|_| ClientEvent::ToggleProofChoice)}></div>
                <div class="modal-content">{self.render_modal_content()}</div>
                <button class="modal-close is-large" aria-label="close" onclick={self.link.callback(|_| ClientEvent::ToggleProofChoice)}></button>
            </div>
        }
    }

    fn render_modal_content(&self) -> Html {
        html! {
            <div class="card">
                <div class="card-content">
                    <div class="content">

                        <h1 class="title is-centered" style="text-align: center">{"Upload proof."}</h1>
                        <div class="file is-large is-boxed" style="display: block">
                            <label class="file-label">
                                <input class="file-input" type="file" name="proof" onchange={self.link.callback(move |value| {

                                    ClientEvent::ProofUpload()
                                })}/>
                                <span class="file-cta">
                                <span class="file-icon">
                                    <i class="fas fa-upload"></i>
                                </span>
                                <span class="file-label">
                                    {"Large file…"}
                                </span>
                                </span>
                            </label>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
