use log::{debug, info, warn};
use yew::prelude::*;
use yew::{html, Component, ComponentLink};

use crate::stellar::*;
use crate::stellar::stellar_data::TOMLCurrency;
use crate::util::badge_check::{self, Badge};

#[derive(Clone, Debug, Eq, PartialEq, Properties)]
pub struct Props {
    pub account: String,
}


#[derive(Clone, Debug, PartialEq, Properties, Default)]
pub struct AccountStorage {
    pub available_badges: Option<Vec<TOMLCurrency>>,
    pub owned_badges: Option<Vec<Badge>>
}

pub struct AccountView {
    link: ComponentLink<AccountView>,
    props: Props,
    status: LoadStatus,
    storage: AccountStorage
}

#[derive(PartialEq, Clone, Debug)]
pub enum LoadStatus {
    Begin,
    FetchAvailableBadges,
    FetchAvailableBadgesDone{available_badges: Vec<TOMLCurrency>},
    FetchOwnedBadges,
    FetchOwnedBadgesDone{owned_badges: Vec<Badge>},
    Done,
    Err(String)
}

impl Component for AccountView {
    type Message = LoadStatus;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link: link,
            props: props,
            status: LoadStatus::Begin,
            storage: AccountStorage::default()
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if !first_render {
            return;
        }

        if !check_valid_public_key(&self.props.account) {
            self.link.send_message(LoadStatus::Err(String::from("Invalid ed25519 public key!")));
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
            },
            LoadStatus::FetchAvailableBadges => {
                self.link.send_future(async {
                    let badges = stellar::fetch_toml_currencies(&String::from(
                        "https://quest.stellar.org/.well-known/stellar.toml",
                    ))
                    .await
                    .unwrap();
                    let badges = badges
                        .into_iter()
                        .filter(|b| b.code.starts_with("SQ")).collect();
                    LoadStatus::FetchAvailableBadgesDone{available_badges: badges}
                });
                false
            },
            LoadStatus::FetchAvailableBadgesDone{available_badges} => {
                self.storage.available_badges = Some(available_badges.clone());
                info!("Loaded available badges: {:?}", available_badges);
                self.link.send_message(LoadStatus::FetchOwnedBadges);
                false
            },
            LoadStatus::FetchOwnedBadges => {
                let pub_key = self.props.account.clone();
                let available_badges = self.storage.available_badges.clone();

                if available_badges.is_none() {
                    warn!("Invalid load state: available badges are None!");
                    self.link.send_message(LoadStatus::Err(String::from("Invalid data received for: available_badges in None!")));
                    return false;
                }

                let available_badges = available_badges.unwrap();

                self.link.send_future(async move {

                    let in_possession = badge_check::fetch_badges(&pub_key, &available_badges).await;

                    LoadStatus::FetchOwnedBadgesDone{owned_badges: in_possession.unwrap_or(vec![])}
                });
                false
            },
            LoadStatus::FetchOwnedBadgesDone{owned_badges} => {
                self.storage.owned_badges = Some(owned_badges.clone());
                info!("Loaded owned badges: {:?}", owned_badges);
                self.link.send_message(LoadStatus::Done);
                false
            },
            LoadStatus::Done => {
                info!("Finished Loading!");
                info!("{:?}", self.storage);
                true
            },
            LoadStatus::Err(_) => true,
            _ => false
        }
    }

    fn change(&mut self, _props: Self::Properties) -> yew::ShouldRender {
        false
    }

    fn view(&self) -> yew::Html {

        match self.status.clone() {
            LoadStatus::Err(msg) => self.view_err(&msg),
            LoadStatus::Done => self.view_account(),
            _ => self.view_loading()
        }
    }
}

impl AccountView {
    fn view_account(&self) -> Html {
        html! {
            <p>{"Account "}{&self.props.account}</p>
        }
    }
    fn view_loading(&self) -> Html {
        html!{
            <p>{"Loading "}{&self.props.account}{"..."}</p>
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


