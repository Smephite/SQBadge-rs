use crate::webpage::pages::{account::AccountView, home::Home, not_found};
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
}

struct Model {}

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _: ComponentLink<Self>) -> Self {
        wasm_logger::init(wasm_logger::Config::default());
        Self {}
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

                <main>
                    <Router<Route> render={Router::render(switch)}/>
                </main>
                <footer class="footer">
                    <div class="content has-text-centered">
                        <p>
                            <strong><a href="https://github.com/Smephite/SQBadge-rss">{"SQBadge-rs"}</a></strong>
                            {" was made with ❤️ and 🍺"}
                        </p>
                    </div>
                </footer>

            </>
        }
    }
}

impl Model {
    fn view_nav(&self) -> Html {
        html! {
            <nav class="navbar is-transparent is-fixed-bottom" role="navigation">
                <div class="navbar-brand">
                    {go_to(Route::Home, html!{
                        <>
                        <span class="is-size-3" style="padding-right: 0px">{"SQBadge"}</span><span style="padding-left: 0px" class="is-size-5">{"-rs"}</span>
                        </>
                    }, vec!["navbar-item", "no-hover"])}
                </div>
            </nav>
        }
    }
}

fn switch(routes: &Route) -> Html {
    match routes {
        Route::Home => html! {<Home />},
        Route::NotFound => not_found::render(),
        Route::Account { id } => html! {<AccountView account={id.clone()}/>},
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
