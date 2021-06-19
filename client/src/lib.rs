mod components;
pub mod idx_store;
pub mod keycloak;

use anyhow::Error;
use components::organisms::NavBar;
use idx_store::IdxDb;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::{IdbRequest, IdbTransaction, IdbTransactionMode};
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
    DBReady(IdxDb),
}

pub struct Model {
    keycloak: Rc<Keycloak>,
    ready: bool,
    _pending_task: Option<FetchTask>,
    _link: ComponentLink<Model>,
    _on_auth_success_handle: CallbackHandle,
    _on_auth_logout_handle: CallbackHandle,
    _insert_request: Option<(
        IdbTransaction,
        IdbRequest,
        Closure<dyn Fn()>,
        Closure<dyn Fn()>,
    )>,
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

        IdxDb::open("demo", 1, |_, db| {
            log::debug!("In here");
            let obj = db.create_object_store("pemento").unwrap();
            log::debug!("Object store pemento created: {:?}", obj);
            let obj = db.create_object_store("chourizo").unwrap();
            log::debug!("Object store chourizo created: {:?}", obj);
        });

        // let transaction: IdbTransaction = db.transaction().unwrap();

        Self {
            keycloak,
            ready: false,
            _pending_task: None,
            _link: link,
            _on_auth_success_handle,
            _on_auth_logout_handle,
            _insert_request: None,
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
            Msg::DBReady(db) => {
                log::debug!("Db Ready");
                log::debug!("Got db {:?}", db);
                let transaction = db
                    .transaction_with_str_and_mode("pemento", IdbTransactionMode::Readwrite)
                    .unwrap();
                let on_success =
                    Closure::wrap(Box::new(move || log::debug!("Succeeeded")) as Box<dyn Fn()>);
                let on_error =
                    Closure::wrap(Box::new(move || log::debug!("Errorreeed")) as Box<dyn Fn()>);
                transaction.set_oncomplete(Some(on_success.as_ref().unchecked_ref()));
                transaction.set_onerror(Some(on_error.as_ref().unchecked_ref()));
                log::debug!("Got transaction {:?}", transaction);
                let obj1 = transaction.object_store("pemento").unwrap();
                log::debug!("Got obj1 {:?}", obj1);
                let insert_request1: IdbRequest = obj1
                    .put_with_key(
                        &JsValue::from_serde("some_key").expect("Serdying"),
                        &JsValue::from_serde("some str").expect("Serdying"),
                    )
                    .expect("error putting");
                log::debug!("Got insert_request1 {:?}", insert_request1);
                log::debug!("Transaction error: {:?}", transaction.error());

                self._insert_request = Some((transaction, insert_request1, on_success, on_error));
                // while insert_request1.ready_state() == IdbRequestReadyState::Pending {}
                // log::debug!("I'm done {:?}", insert_request1.result());
                false
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
