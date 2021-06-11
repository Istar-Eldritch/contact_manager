use std::rc::Rc;

use yew::prelude::*;

use crate::{components::molecules::AuthProfile, keycloak::Keycloak};

pub enum Msg {}

pub struct NavBar {
    keycloak: Rc<Keycloak>,
}

#[derive(Properties, Clone)]
pub struct NavBarProps {
    pub keycloak: Rc<Keycloak>,
}

impl Component for NavBar {
    type Message = Msg;
    type Properties = NavBarProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            keycloak: props.keycloak,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="navbar">
                <AuthProfile keycloak=self.keycloak.clone()/>
            </div>
        }
    }
}
