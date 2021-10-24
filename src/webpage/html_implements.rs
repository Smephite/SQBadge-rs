use yew::{html, Html};

use crate::{util::badge_check::Badge, webpage::components::badge::BadgeCard};

impl Into<Html> for Badge {
    fn into(self) -> Html {
        html! {
            <BadgeCard badge={self} valid={true}/>
        }
    }
}
