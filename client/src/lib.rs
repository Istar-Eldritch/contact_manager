mod components;
pub mod idx_store;
pub mod keycloak;

use std::rc::Rc;

use anyhow::Error;
use components::organisms::NavBar;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{convert::IntoWasmAbi, prelude::Closure, JsCast, JsValue};
use web_sys::{
    window, IdbDatabase, IdbFactory, IdbOpenDbRequest, IdbRequest, IdbRequestReadyState,
    IdbTransaction, IdbTransactionMode,
};
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
    DBReady,
}

pub struct Model {
    keycloak: Rc<Keycloak>,
    ready: bool,
    _pending_task: Option<FetchTask>,
    _link: ComponentLink<Model>,
    _on_auth_success_handle: CallbackHandle,
    _on_auth_logout_handle: CallbackHandle,
    _db_callback: Closure<dyn Fn()>,
    _on_upgrade_callback: Closure<dyn Fn()>,
    _insert_request: Option<(
        IdbTransaction,
        IdbRequest,
        Closure<dyn Fn()>,
        Closure<dyn Fn()>,
    )>,
    db: IdbOpenDbRequest,
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

        let factory: IdbFactory = window().unwrap().indexed_db().unwrap().unwrap();

        let db = factory.open("demo").unwrap();

        let db_ready_cb = link.callback(|_| Msg::DBReady);

        let callback = Closure::wrap(Box::new(move || {
            log::debug!("Loaded");
            db_ready_cb.emit(());
        }) as Box<dyn Fn()>);

        db.set_onsuccess(Some(&callback.as_ref().unchecked_ref()));

        let db_request = db.clone();

        let _on_upgrade_callback = Closure::wrap(Box::new(move || {
            log::debug!("Loaded");
            let db: IdbDatabase = db_request.result().unwrap().unchecked_into();
            let obj = db.create_object_store("pemento").unwrap();
            log::debug!("Object store pemento created: {:?}", obj);
            let obj = db.create_object_store("chourizo").unwrap();
            log::debug!("Object store chourizo created: {:?}", obj);
        }) as Box<dyn Fn()>);

        db.set_onupgradeneeded(Some(&_on_upgrade_callback.as_ref().unchecked_ref()));

        // let transaction: IdbTransaction = db.transaction().unwrap();

        Self {
            keycloak,
            ready: false,
            _pending_task: None,
            _link: link,
            _on_auth_success_handle,
            _on_auth_logout_handle,
            _db_callback: callback,
            _on_upgrade_callback,
            _insert_request: None,
            db,
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
            Msg::DBReady => {
                log::debug!("Db Ready");
                let db: IdbDatabase = self.db.result().unwrap().unchecked_into();
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
