use crate::util::badge_check::Badge;

use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub badge: Badge,
}

pub struct BadgeCard {
    pub badge: Badge,
}

impl Component for BadgeCard {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self {
            badge: props.badge.to_owned(),
        }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let mut cls = vec!["badge"];
        if !self.badge.owned {
            cls.push("disabled");
        }

        let mut name = self.badge.token.code.clone();
        let mut monochrome = "";
        if &self.badge.token.tag == "mono" {
            name.push_str(" mono");
            monochrome = "(monochrome) "
        }

        let inner = html! {
            <>
                <img style="margin-left: auto; margin-right: auto; display: block;" src={self.badge.token.image.clone()}
                 title={
                     match self.badge.date_accuired.clone() {
                         Some(date) => format!("{} {}owned since {}", &self.badge.token.code, monochrome, date),
                         None => format!("{} not accuired yet", &name)
                     }
                 } alt="" />
                <p class="badge-name">{&name}</p>
            </>
        };
        if self.badge.owned {
            html! {
                <div class={classes!(cls)}>
                    <a href={format!("https://horizon.stellar.org/transactions/{}", self.badge.tx_hash.clone().unwrap())}>{ inner }</a>
                </div>
            }
        } else {
            html! {
                <div class={classes!(cls)}>
                    { inner }
                </div>
            }
        }
    }
}
