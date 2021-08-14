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
        html! {
            <div class={classes!(cls)}>
                <img src={self.badge.token.image.clone()}/>
                <p>{self.badge.token.code.clone()}</p>
            </div>
        }
    }
}
