use crate::webpage::pages::{account::AccountView, home::Home, not_found, proof::ProofVerify};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(::yew_router::Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/404")]
    #[not_found]
    NotFound,
    #[at("/account/:id")]
    Account { id: String },
    #[at("/proof/:id")]
    Proof { id: String },
}

struct Model {
    link: ComponentLink<Model>,
}

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        wasm_logger::init(wasm_logger::Config::default());
        Self { link: link }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>

            { self.view_nav() }
            <div class="column-design">
                <main>
                    <Router<Route> render={Router::render(switch)}/>
                </main>
                <footer class="footer">
                    <div class="content has-text-centered">
                        <p>
                            <strong><a href="https://github.com/Smephite/SQBadge-rs">{"SQBadge-rs"}</a></strong>
                            {" was made with ❤️ and 🍺"}
                        </p>
                    </div>
                </footer>
            </div>
            </>
        }
    }
}

impl Model {
    fn view_nav(&self) -> Html {
        html! {
                <div style="position: fixed; bottom: 0.5rem; left: 0.75rem;">
                    {go_to(Route::Home, html!{
                        <>
                        <span class="is-size-3" style="padding-right: 0px">{"SQBadge"}</span><span style="padding-left: 0px; position: absolute; bottom: 0.5rem" class="is-size-5" >{"-rs"}</span>
                        </>
                    }, vec!["is-shadowless", "has-text-dark"])}
                </div>
        }
    }
}
fn switch(routes: &Route) -> Html {
    match routes {
        Route::Home => html! {<Home />},
        Route::NotFound => not_found::render(),
        Route::Account { id } => html! {<AccountView account={id.clone()}/>},
        Route::Proof { id } => html! {<ProofVerify proof={id.clone()}/>},
    }
}

pub fn go_to(route: Route, html: Html, classes: Vec<&str>) -> Html {
    html! {
        <Link<Route> route={route} classes={classes!(classes.iter().map(|&s| String::from(s)).collect::<Vec<String>>())}>{html}</Link<Route>>
    }
}

pub fn start() {
    let document = yew::utils::document();
    let element = document.query_selector("#app").unwrap().unwrap();
    yew::start_app_in_element::<Model>(element);
}
