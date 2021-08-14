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

        let inner = html!{
            <>
                <img src={self.badge.token.image.clone()}/>
                <p>{self.badge.token.code.clone()}</p>
            </>
        };
        if self.badge.owned {
            
            html! {
                <div class={classes!(cls)}>
                    <a href={format!("https://horizon.stellar.org/transactions/{}", self.badge.tx_hash)}>{ inner }</a>
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
