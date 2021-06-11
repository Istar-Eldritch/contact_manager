mod components;
pub mod keycloak;

use std::rc::Rc;

use components::organisms::NavBar;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::Closure;
use yew::prelude::*;
use yewtil::future::LinkFuture;

use keycloak::{CallbackHandle, Keycloak, KeycloakConfig};

pub enum Msg {
    KeycloakInitialized,
    KeycloakStateChanged,
}

pub struct Model {
    keycloak: Rc<Keycloak>,
    ready: bool,
    _link: ComponentLink<Model>,
    _on_auth_success_handle: CallbackHandle,
    _on_auth_logout_handle: CallbackHandle,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserAttributes {}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let config = KeycloakConfig {
            url: String::from("http://localhost:8081/auth"),
            realm: String::from("demo"),
            client_id: String::from("frontend"),
        };
        let keycloak = Rc::new(Keycloak::new(config));

        let kc = keycloak.clone();
        link.send_future(async move {
            // TODO: Handle initialization error
            kc.init().await.unwrap();
            Msg::KeycloakInitialized
        });

        let on_auth_success_cb = link.callback(|_| Msg::KeycloakStateChanged);
        let cb = Closure::wrap(Box::new(move || {
            on_auth_success_cb.emit(());
        }) as Box<dyn Fn()>);
        let _on_auth_success_handle = keycloak.on_auth_success(cb);

        let on_auth_logout_cb = link.callback(|_| Msg::KeycloakStateChanged);
        let cb = Closure::wrap(Box::new(move || {
            on_auth_logout_cb.emit(());
        }) as Box<dyn Fn()>);
        let _on_auth_logout_handle = keycloak.on_auth_logout(cb);

        Self {
            keycloak,
            ready: false,
            _link: link,
            _on_auth_success_handle,
            _on_auth_logout_handle,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::KeycloakInitialized => {
                log::debug!("Keycloak initialized");
                self.ready = true;
                true
            }
            Msg::KeycloakStateChanged => {
                log::debug!("Keycloak state udpated: {}", self.keycloak.authenticated());
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <NavBar keycloak={self.keycloak.clone()}/>
            </div>
        }
    }
}
