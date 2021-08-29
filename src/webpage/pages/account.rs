use log::{debug, info, warn};
use yew::prelude::*;
use yew::{html, Component, ComponentLink};

use crate::stellar::stellar_data::TOMLCurrency;
use crate::stellar::*;
use crate::util::badge_check::{self, Badge};
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
    status: LoadStatus,
    storage: AccountStorage,
}

#[derive(PartialEq, Clone, Debug)]
pub enum LoadStatus {
    Begin,
    FetchAvailableBadges,
    FetchAvailableBadgesDone { available_badges: Vec<TOMLCurrency> },
    FetchOwnedBadges,
    FetchOwnedBadgesDone { owned_badges: Vec<Badge> },
    Done,
    Err(String),
}

impl Component for AccountView {
    type Message = LoadStatus;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link: link,
            props: props,
            status: LoadStatus::Begin,
            storage: AccountStorage::default(),
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if !first_render {
            return;
        }

        if !check_valid_public_key(&self.props.account) {
            self.link
                .send_message(LoadStatus::Err(String::from("Invalid ed25519 public key!")));
            return;
        }
        self.link.send_message(LoadStatus::Begin);
    }

    fn update(&mut self, status: Self::Message) -> yew::ShouldRender {
        self.status = status.clone();
        debug!("LoadStatus: {:?}", status);
        match status {
            LoadStatus::Begin => {
                self.link.send_message(LoadStatus::FetchAvailableBadges);
                false
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
                self.storage.available_badges = Some(available_badges.clone());
                info!("Loaded available badges: {:?}", available_badges);
                self.link.send_message(LoadStatus::FetchOwnedBadges);
                false
            }
            LoadStatus::FetchOwnedBadges => {
                let pub_key = self.props.account.clone();
                let available_badges = self.storage.available_badges.clone();

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
                self.storage.owned_badges = Some(owned_badges.clone());
                info!("Loaded owned badges: {:?}", owned_badges);
                self.link.send_message(LoadStatus::Done);
                false
            }
            LoadStatus::Done => {
                info!("Finished Loading!");
                info!("{:?}", self.storage);
                true
            }
            LoadStatus::Err(_) => true,
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
                <h2 class="title" style="text-align: center">
                    {"Account "}
                    <a href={format!("https://stellar.expert/explorer/public/account/{}", &self.props.account)}>
                        {&self.props.account}
                    </a>
                </h2>
                <p style="text-align: center">
                    {format!(" Completed {}/{} Quests", completed_num, badges_num)}
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
            _ => String::default(),
        };
        info! {"{:?}", status};
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
            <p>{"Error: "}{message}</p>
        }
    }
}

fn check_valid_public_key(_: &String) -> bool {
    true
}
