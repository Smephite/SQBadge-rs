use log::{debug, info, warn};
use yew::prelude::*;
use yew::{html, Component, ComponentLink};

use crate::js::albedo;
use crate::stellar::stellar_data::TOMLCurrency;
use crate::stellar::*;
use crate::util::badge_check::{self, Badge};
use crate::util::proof_encoding::{self, Proof};
use crate::webpage::components::badge::BadgeCard;
use crate::webpage::html_implements;
use itertools::Itertools;

#[derive(Clone, Debug, Eq, PartialEq, Properties)]
pub struct Props {
    pub proof: String,
}

#[derive(Clone, Debug, PartialEq, Properties, Default)]
pub struct ProofStorage {
    pub available_badges: Option<Vec<TOMLCurrency>>,
    pub proof_claim: Option<Proof>,
    pub owned_badges: Option<Vec<Badge>>,
    pub account: Option<String>,
    pub valid: bool,
}

pub struct ProofVerify {
    link: ComponentLink<ProofVerify>,
    props: Props,
    status: LoadStatus,
    proof: ProofStorage,
    decoded_proof: Option<(bool, String, String)>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum LoadStatus {
    Begin,
    FetchAvailableBadges,
    FetchAvailableBadgesDone { available_badges: Vec<TOMLCurrency> },
    CheckProof,
    FetchOwnedBadges,
    FetchOwnedBadgesDone { owned_badges: Vec<Badge> },
    Done,
    Err(String),
    None,
}

impl Component for ProofVerify {
    type Message = LoadStatus;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link: link,
            props: props,
            status: LoadStatus::None,
            proof: ProofStorage::default(),
            decoded_proof: None,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if !first_render {
            return;
        }

        self.link.send_message(LoadStatus::Begin);
    }

    fn update(&mut self, status: Self::Message) -> yew::ShouldRender {
        self.status = status.clone();
        debug!("LoadStatus: {:?}", status);
        match status {
            LoadStatus::Begin => {
                self.decrypt_proof();
                self.link.send_message(LoadStatus::FetchAvailableBadges);
                true
            }
            LoadStatus::FetchAvailableBadges => {
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
                    LoadStatus::FetchAvailableBadgesDone {
                        available_badges: badges,
                    }
                });
                false
            }
            LoadStatus::FetchAvailableBadgesDone { available_badges } => {
                self.proof.available_badges = Some(available_badges.clone());
                info!("Loaded available badges: {:?}", available_badges);
                self.link.send_message(LoadStatus::CheckProof);
                false
            }
            LoadStatus::CheckProof => {
                let err_decrypting = !self.decrypt_badges();

                if err_decrypting {
                    self.link
                        .send_message(LoadStatus::Err(String::from("Invalid proof!")));
                } else {
                    self.link.send_message(LoadStatus::FetchOwnedBadges);
                }
                false
            }
            LoadStatus::FetchOwnedBadges => {
                let pub_key = self.proof.account.clone().unwrap().clone();
                let available_badges = self.proof.available_badges.clone();

                if available_badges.is_none() {
                    warn!("Invalid load state: available badges are None!");
                    self.link.send_message(LoadStatus::Err(String::from(
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
                        return LoadStatus::Err(format!("Error: {:?}", in_possession.err()));
                    }

                    LoadStatus::FetchOwnedBadgesDone {
                        owned_badges: in_possession.unwrap(),
                    }
                });
                false
            }
            LoadStatus::FetchOwnedBadgesDone { owned_badges } => {
                self.proof.owned_badges = Some(owned_badges.clone());
                info!("Loaded owned badges: {:?}", owned_badges);
                self.link.send_message(LoadStatus::Done);
                false
            }
            LoadStatus::Done => {
                info!("Finished Loading!");
                info!("{:?}", self.proof);
                true
            }
            _ => true,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> yew::ShouldRender {
        false
    }

    fn view(&self) -> yew::Html {
        match self.status.clone() {
            LoadStatus::Err(msg) => self.view_err(&msg),
            LoadStatus::Done => self.view_account(),
            other => self.view_loading(other),
        }
    }
}

impl ProofVerify {
    fn render_series(&self, series: &String, badges: &Vec<Badge>) -> Html {
        let claimed_owned_badges = self
            .proof
            .proof_claim
            .clone()
            .unwrap()
            .owned_badges
            .into_iter()
            .map(|t| t.code)
            .collect::<Vec<String>>();

        let colored_badges = badges
            .clone()
            .into_iter()
            .filter(|b| b.owned)
            .chain(badges.clone().into_iter().filter(|b| !b.is_mono()))
            .unique_by(|b| b.token.code.clone())
            .sorted_by(|a, b| a.token.code.cmp(&b.token.code))
            .map(|b| -> Html{
                html!{
                    <BadgeCard badge={b.clone()} valid={!(claimed_owned_badges.contains(&b.token.code) ^ b.owned)}/>
                }
            })
            .collect::<Html>();

        html! {
            <section class="section">
            <h1 class="title" style="text-align: center">{series}</h1>
            {
                colored_badges
            }
            </section>
        }
    }
    fn view_account(&self) -> Html {
        let owned_num = self
            .proof
            .owned_badges
            .clone()
            .unwrap_or(vec![])
            .into_iter()
            .filter(|b| b.owned)
            .count();
        let completed_num = self
            .proof
            .owned_badges
            .clone()
            .unwrap_or(vec![])
            .into_iter()
            .filter(|b| b.owned)
            .unique_by(|b| b.token.code.clone())
            .count();
        let badges_num = self
            .proof
            .owned_badges
            .clone()
            .unwrap_or(vec![])
            .into_iter()
            .unique_by(|b| b.token.code.clone())
            .count();

        let claimed_num = self
            .proof
            .proof_claim
            .clone()
            .unwrap_or(Proof::default())
            .owned_badges
            .into_iter()
            .unique_by(|b| b.code.clone())
            .count();

        html! {
            <>
                <h2 class="title mid-center" style="text-align: center">
                    {"Account "}
                    <a href={format!("https://stellar.expert/explorer/public/account/{}", &self.proof.account.clone().unwrap())}>
                        {&self.proof.account.clone().unwrap()}
                    </a>
                </h2>
                <p style="text-align: center" class="mid-center">
                    {format!(" Completed {}/{} Quests", completed_num, badges_num)}
                    {
                        if owned_num > badges_num {
                            format!(
                             " (Owns {} / {} including mono badges)",
                             owned_num,
                             self.proof.owned_badges.clone().unwrap_or(vec![]).len())
                        } else {
                            "".to_string()
                        }
                    }
                </p>

                <p style="text-align: center; color:red" class="mid-center" hidden={claimed_num == owned_num}>
                    {format!("Invalid Proof! Claimed to have completed {} quests!", claimed_num)}
                </p>

                <div class="badges">
                {
                    self.proof.owned_badges.clone()
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
                        .map(|(series, badges)|self.render_series(&series, &badges.collect()))
                        .collect::<Html>()
                }
                </div>
            </>
        }
    }
    fn view_loading(&self, status: LoadStatus) -> Html {
        let description = match status {
            LoadStatus::Begin
            | LoadStatus::FetchAvailableBadges
            | LoadStatus::FetchAvailableBadgesDone {
                available_badges: _,
            } => String::from("Fetching all available badges..."),
            LoadStatus::FetchOwnedBadges | LoadStatus::FetchOwnedBadgesDone { owned_badges: _ } => {
                String::from("Verifying users badges...")
            }
            _ => String::from("unknown"),
        };
        info! {"{:?}", status};
        html! {
            <div class="container is-max-desktop">
                <div class="sqb-centered">
                    <h2 class="subtitle is-centered">{"Verifying Proof for "} <i>{&self.proof.account.clone().unwrap_or(String::from("unknown"))}</i></h2>
                    { if description != String::default() { &description } else {""}}
                </div>
            </div>
        }
    }
    fn view_err(&self, message: &String) -> Html {
        html! {
            <p>{"Error: "}{message}</p>
        }
    }

    fn decrypt_proof(&mut self) -> bool {
        let proof = proof_encoding::verify_albedo_signed_message(&self.props.proof);

        if proof.is_none() {
            return false;
        }

        let proof = proof.unwrap();

        self.decoded_proof = Some(proof.clone());

        self.proof.valid = proof.0;
        self.proof.account = Some(proof.2);

        return true;
    }

    fn decrypt_badges(&mut self) -> bool {
        let proof = self.decoded_proof.clone();
        if self.proof.available_badges.is_none() {
            return false;
        }
        if proof.is_none() {
            return false;
        }

        let proof = proof.unwrap();

        let decrypted_badges = proof_encoding::Proof::decode_v1(
            &proof.1,
            &self.proof.available_badges.clone().unwrap(),
        );

        if decrypted_badges.is_err() {
            return false;
        }
        let decrypted_badges = decrypted_badges.unwrap();

        debug!("Proof claims ownership over: {:?}", decrypted_badges);
        self.proof.proof_claim = Some(decrypted_badges);
        return true;
    }
}
