mod components;
pub mod keycloak;

use std::rc::Rc;

use anyhow::Error;
use components::organisms::NavBar;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::Closure;
use yew::{
    format::Nothing,
    prelude::*,
    services::{
        fetch::{FetchTask, Request, Response},
        FetchService,
    },
};
use yewtil::future::LinkFuture;

use keycloak::{CallbackHandle, Keycloak, KeycloakConfig};

use crate::components::atoms::Button;

pub enum Msg {
    KeycloakInitialized,
    KeycloakStateChanged,
    ClickButton,
    Response,
}

pub struct Model {
    keycloak: Rc<Keycloak>,
    ready: bool,
    _pending_task: Option<FetchTask>,
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
            _pending_task: None,
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
            Msg::ClickButton => {
                let get_request = Request::get("http://localhost:8083/")
                    .header(
                        "Authorization",
                        format!("Bearer {}", self.keycloak.token().unwrap()),
                    )
                    .body(Nothing)
                    .unwrap();
                let task = FetchService::fetch(
                    get_request,
                    self._link
                        .callback(|_r: Response<Result<String, Error>>| Msg::Response),
                )
                .unwrap();
                self._pending_task = Some(task);
                true
            }
            Msg::Response => {
                self._pending_task = None;
                log::debug!("Response!");
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let click_cb = self._link.callback(|_| Msg::ClickButton);
        html! {
            <div>
                <NavBar keycloak={self.keycloak.clone()}/>
                <Button disabled={self._pending_task.is_some()} onclick={click_cb}>{
                    if self._pending_task.is_some() {
                        html! {
                            <i class="fas fa-sync"></i>
                        }
                    } else {
                        html! {
                            {"Request"}
                        }
                    }
                }</Button>
            </div>
        }
    }
}
