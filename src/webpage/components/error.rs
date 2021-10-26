use crate::util::badge_check::Badge;

use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub message: String,
}

pub struct ErrorCard {
    pub message: String,
}

impl Component for ErrorCard {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Self {
            message: props.message,
        }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
           <div class="sqb-centered" style="display: flex; justify-content: center">
               <div class="notification is-danger" style="width: auto">
                   <p style="font-size: 1.5rem; font-weight: 500;">{"Error:"}</p>
                   <p style="text-align: center; ">{&self.message}</p>
               </div>
           </div>
        }
    }
}
