use chrono::Utc;
use js_sys::JsString;
use log::{debug, warn};
use serde_json::Value;
use wasm_bindgen::JsValue;
use yew::prelude::*;
use yew::{html, Component, ComponentLink};

use base64;

use crate::js::albedo;
use crate::stellar::stellar_data::TOMLCurrency;
use crate::stellar::*;
use crate::util::badge_check::{self, Badge};
use crate::util::error::{Error, StellarErr};
use crate::util::proof_encoding::Proof;
use crate::webpage::components::error::ErrorCard;
use itertools::Itertools;

#[derive(Clone, Debug, Eq, PartialEq, Properties)]
pub struct Props {
    pub account: String,
}

#[derive(Clone, Debug, PartialEq, Properties, Default)]
pub struct AccountStorage {
    pub available_badges: Option<Vec<TOMLCurrency>>,
    pub owned_badges: Option<Vec<Badge>>,
}

pub struct AccountView {
    link: ComponentLink<AccountView>,
    props: Props,
    status: WorkFunction,
    storage: AccountStorage,
    signing_message: String,
    modal_shown: bool,
}

#[derive(PartialEq, Clone, Debug)]
pub enum WorkFunction {
    Begin,
    FetchAvailableBadges,
    FetchAvailableBadgesDone { available_badges: Vec<TOMLCurrency> },
    FetchOwnedBadges,
    FetchOwnedBadgesDone { owned_badges: Vec<Badge> },
    Done,
    None,
    ToggleModal,
    ModalProofTextChange(String),
    CreateProof,
    ProofSignDone(Result<JsValue, JsValue>),
    Err(String),
}

impl Component for AccountView {
    type Message = WorkFunction;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link: link,
            props: props,
            status: WorkFunction::Begin,
            storage: AccountStorage::default(),
            signing_message: String::new(),
            modal_shown: false,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if !first_render {
            return;
        }

        if !check_valid_public_key(&self.props.account) {
            self.link.send_message(WorkFunction::Err(String::from(
                "Invalid ed25519 public key!",
            )));
            return;
        }
        self.link.send_message(WorkFunction::Begin);
    }

    fn update(&mut self, status: Self::Message) -> yew::ShouldRender {
        self.status = status.clone();
        debug!("LoadStatus: {:?}", status);
        match status {
            WorkFunction::Begin => {
                self.link.send_message(WorkFunction::FetchAvailableBadges);
                false
            }
            WorkFunction::FetchAvailableBadges => {
                self.link.send_future(async {
                    let badges = stellar::fetch_toml_currencies(&String::from(
                        "https://quest.stellar.org/.well-known/stellar.toml",
                    ))
                    .await
                    .unwrap();
                    let badges = badges
                        .into_iter()
                        .filter(|b| b.code.starts_with("SQ") || b.code.starts_with("SSQ"))
                        .collect();
                    WorkFunction::FetchAvailableBadgesDone {
                        available_badges: badges,
                    }
                });
                false
            }
            WorkFunction::FetchAvailableBadgesDone { available_badges } => {
                self.storage.available_badges = Some(available_badges.clone());
                debug!("Loaded available badges: {:?}", available_badges);
                self.link.send_message(WorkFunction::FetchOwnedBadges);
                false
            }
            WorkFunction::FetchOwnedBadges => {
                let pub_key = self.props.account.clone();
                let available_badges = self.storage.available_badges.clone();

                if available_badges.is_none() {
                    warn!("Invalid load state: available badges are None!");
                    self.link.send_message(WorkFunction::Err(String::from(
                        "Invalid data received for: available_badges in None!",
                    )));
                    return false;
                }

                let available_badges = available_badges.unwrap();

                self.link.send_future(async move {
                    let in_possession =
                        badge_check::fetch_badges(&pub_key, &available_badges).await;

                    if in_possession.is_err() {
                        // Sth went wrong fetching --> probably wrong account id (if not handled inbefore ._.)
                        let err =  in_possession.err().unwrap();

                        if let Error::StellarErr(s_err) = err  {
                            return WorkFunction::Err(match s_err {
                                StellarErr::AccountNotFound => {
                                    format!("The account you specified could not be found!")
                                },
                                StellarErr::InvalidPublicKey => {
                                    format!("The specified public key is not in a valid ed25519 format!")
                                },
                                _ => {
                                    format!("Unknown error while trying to connect to the stellar network!")
                                }

                            });
                        }

                        return WorkFunction::Err(format!("{:?}", err));
                    }

                    WorkFunction::FetchOwnedBadgesDone {
                        owned_badges: in_possession.unwrap(),
                    }
                });
                false
            }
            WorkFunction::FetchOwnedBadgesDone { owned_badges } => {
                self.storage.owned_badges = Some(owned_badges.clone());
                debug!("Loaded owned badges: {:?}", owned_badges);
                self.link.send_message(WorkFunction::Done);
                false
            }
            WorkFunction::Done => {
                debug!("Finished Loading!");
                debug!("{:?}", self.storage);
                true
            }
            WorkFunction::ToggleModal => {
                self.modal_shown = !self.modal_shown;

                self.signing_message = String::new();
                self.status = WorkFunction::Done;
                true
            }
            WorkFunction::ModalProofTextChange(msg) => {
                self.signing_message = msg;
                false
            }
            WorkFunction::CreateProof => {
                let mut proof = Proof::default();
                proof.timestamp = Some(Utc::now().timestamp());
                proof.unique_id = match self.signing_message.len() == 0 {
                    true => None,
                    _ => Some(self.signing_message.clone()),
                };

                proof.owned_badges = self
                    .storage
                    .owned_badges
                    .clone()
                    .unwrap_or(vec![])
                    .into_iter()
                    .filter(|b| b.owned)
                    .map(|b| b.token.clone())
                    .collect();
                let data = proof.encode_v1();
                if data.is_ok() {
                    let data = data.unwrap();
                    let pub_key = self.props.account.clone();
                    self.link.send_future(async {
                        let albedo_response = unsafe {
                            albedo::sign_message_pubkey(
                                JsString::from(data),
                                JsString::from(pub_key),
                            )
                            .await
                        };
                        WorkFunction::ProofSignDone(albedo_response)
                    });
                }
                self.modal_shown = false;
                self.status = WorkFunction::Done;
                true
            }
            WorkFunction::ProofSignDone(response) => {
                if response.is_ok() {
                    debug!("{:?}", response);
                } else {
                    warn!("{:?}", response);
                }
                true
            }
            WorkFunction::None => false,
            WorkFunction::Err(_) => true,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> yew::ShouldRender {
        false
    }

    fn view(&self) -> yew::Html {
        match self.status.clone() {
            WorkFunction::Err(msg) => self.view_err(&msg),
            WorkFunction::Done | WorkFunction::ProofSignDone(_) => self.view_account(),
            other => self.view_loading(other),
        }
    }
}

fn render_series(series: &String, badges: &Vec<Badge>) -> Html {
    html! {
        <section class="section">
        <h1 class="title" style="text-align: center">{series}</h1>
        {
            badges.clone().into_iter()
            .filter(|b| b.owned)
            .chain(badges.clone().into_iter().filter(|b| !b.is_mono()))
            .unique_by(|b| b.token.code.clone())
            .sorted_by(|a, b| a.token.code.cmp(&b.token.code))
            .collect::<Html>()}
        </section>
    }
}

impl AccountView {
    fn view_account(&self) -> Html {
        let owned_num = self
            .storage
            .owned_badges
            .clone()
            .unwrap_or(vec![])
            .into_iter()
            .filter(|b| b.owned)
            .count();
        let completed_num = self
            .storage
            .owned_badges
            .clone()
            .unwrap_or(vec![])
            .into_iter()
            .filter(|b| b.owned)
            .unique_by(|b| b.token.code.clone())
            .count();
        let badges_num = self
            .storage
            .owned_badges
            .clone()
            .unwrap_or(vec![])
            .into_iter()
            .unique_by(|b| b.token.code.clone())
            .count();

        html! {
            <>
                <h2 class="title mid-center" style="text-align: center">
                    {"Account "}
                    <a href={format!("https://stellar.expert/explorer/public/account/{}", &self.props.account)}>
                        {&self.props.account}
                    </a>
                </h2>
                <p style="text-align: center" class="mid-center">
                    {format!(" Earned {}/{} Badges", completed_num, badges_num)}
                    {
                        if owned_num > badges_num {
                            format!(
                             " (Owns {} / {} including mono badges)",
                             owned_num,
                             self.storage.owned_badges.clone().unwrap_or(vec![]).len())
                        } else {
                            "".to_string()
                        }
                    }
                </p>
                {
                    if let WorkFunction::ProofSignDone(_) = self.status.clone() {
                        self.view_proof_sign_response()
                    } else {
                        "".to_string().into()
                    }
                }
                <div class="badges">
                {
                    self.storage.owned_badges.clone()
                        .unwrap_or(vec![]).into_iter()
                        .group_by(|badge| {
                            let mut series = badge.token.code.clone();
                            if series.starts_with("SSQ") {
                                series.truncate(3);
                            } else {
                                series.truncate(4);
                            }
                            series
                        }).into_iter()
                        .map(|(series, badges)|render_series(&series, &badges.collect()))
                        .collect::<Html>()
                }
                </div>
                <button onclick={self.link.callback(|_| WorkFunction::ToggleModal)} class="button is-floating is-primary">
                    <i class="fas fa-key"></i>
                </button>
                {
                    if self.modal_shown {
                        self.render_modal()
                    } else {
                        Html::default()
                    }
                }
            </>
        }
    }

    fn view_proof_sign_response(&self) -> Html {
        if let WorkFunction::ProofSignDone(response) = self.status.clone() {
            let message: String;
            let message_header: String;
            let class: String;

            match response {
                Ok(resp) => {
                    let mut message_opt: Option<String> = None;
                    let mut message_class_opt: Option<String> = None;
                    let mut message_header_opt: Option<String> = None;
                    if let Ok(resp) = resp.into_serde::<Value>() {
                        let resp = resp.as_object().unwrap(); // todo make safe again

                        if resp.get("intent").unwrap().as_str() == Some("sign_message") {
                            let _included_message = resp.get("message").unwrap().as_str().unwrap();
                            let message_signature =
                                resp.get("message_signature").unwrap().as_str().unwrap();
                            let pub_key = resp.get("pubkey").unwrap().as_str().unwrap();
                            let signed_message =
                                resp.get("signed_message").unwrap().as_str().unwrap();

                            if pub_key == self.props.account {
                                message_header_opt = Some(String::from("Success"));
                                message_class_opt = Some("is-success".to_string());

                                let interesting =
                                    format!("{}:{}", message_signature, signed_message);

                                message_opt = Some(base64::encode(interesting));
                            } else {
                                message_opt = Some("Signing key does not match!".to_string());
                            }
                        } else {
                            message_opt = Some("Wrong intent!".to_string());
                        }
                    }

                    class = message_class_opt.unwrap_or("is-danger".to_string());
                    message_header = message_header_opt.unwrap_or("Error".to_string());
                    message = message_opt.unwrap_or("unknown".to_string());
                }
                Err(err) => {
                    let err: serde_json::Result<serde_json::Value> = err.into_serde();

                    let get_err =
                        |err: serde_json::Result<serde_json::Value>| -> serde_json::Result<String> {
                            let err = err?;

                            let err = err.as_object().unwrap(); // todo make safe again

                            let err_msg = err.get("message").unwrap().as_str().unwrap();

                            Ok(format!("Albedo: {}", err_msg))
                        };

                    if let Ok(err) = get_err(err) {
                        message = err;
                    } else {
                        message = "unknown".to_string();
                    }

                    message_header = String::from("Signing Error");
                    class = "is-danger".to_string();
                }
            };

            let classes = vec!["message".to_string(), "mid-center".to_string(), class];

            return html! {
                <article class={classes} style="margin-top: 1.5rem; margin-bottom: 0">
                    <div class="message-header">
                        <p>{message_header}</p>
                    </div>
                    <div class="message-body" style="word-break: break-all;">
                        {message}
                    </div>
                </article>
            };
        }

        return html!("");
    }

    fn view_loading(&self, status: WorkFunction) -> Html {
        let description = match status {
            WorkFunction::Begin
            | WorkFunction::FetchAvailableBadges
            | WorkFunction::FetchAvailableBadgesDone {
                available_badges: _,
            } => String::from("Fetching all available badges..."),
            WorkFunction::FetchOwnedBadges
            | WorkFunction::FetchOwnedBadgesDone { owned_badges: _ } => {
                String::from("Verifying users badges...")
            }
            _ => String::default(),
        };
        debug! {"{:?}", status};
        html! {
            <div class="container is-max-desktop">
                <div class="sqb-centered">
                    <h2 class="subtitle is-centered">{"Loading Badges for "} <i>{&self.props.account}</i></h2>
                    { if description != String::default() { &description } else {""}}
                </div>
            </div>
        }
    }
    fn view_err(&self, message: &String) -> Html {
        html! {
            <ErrorCard message={message.clone()} />
        }
    }
    fn render_modal(&self) -> Html {
        html! {
            <div class="modal is-active">
                <div class="modal-background" onclick={self.link.callback(|_| WorkFunction::ToggleModal)}></div>
                <div class="modal-content">{self.render_modal_content()}</div>
                <button class="modal-close is-large" aria-label="close" onclick={self.link.callback(|_| WorkFunction::ToggleModal)}></button>
            </div>
        }
    }

    fn render_modal_content(&self) -> Html {
        let proof_text_change = self
            .link
            .callback(|e: InputData| WorkFunction::ModalProofTextChange(e.value));
        html! {
            <div class="card">
                <div class="card-content">
                    <div class="content">

                        <h1 class="title is-centered" style="text-align: center">{"Specify proof message."}</h1>
                        <textarea class="textarea" placeholder="Enter message..." name="proof" oninput={proof_text_change}/>
                        <div class="mt-1" style="display: flex; justify-content: flex-end">
                            <button class="button is-primary" onclick={self.link.callback(|_| WorkFunction::CreateProof)}>{"Sign"}</button>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}

fn check_valid_public_key(_: &String) -> bool {
    true
}
