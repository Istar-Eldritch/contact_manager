use std::rc::Rc;

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yewtil::future::LinkFuture;

use crate::{
    components::atoms::{Button, Dropdown},
    keycloak::{Keycloak, UserProfile},
    UserAttributes,
};

pub enum Msg {
    Login,
    Logout,
    ToggleDropdown,
    UserProfileLoaded(UserProfile<UserAttributes>),
}

pub struct AuthProfile {
    keycloak: Rc<Keycloak>,
    link: ComponentLink<AuthProfile>,
    user: Option<UserProfile<UserAttributes>>,
    dropdown_open: bool,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub keycloak: Rc<Keycloak>,
}

impl Component for AuthProfile {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            keycloak: props.keycloak,
            link,
            user: None,
            dropdown_open: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Login => {
                let kc = self.keycloak.clone();

                spawn_local(async move {
                    // TODO: handle login error
                    kc.login().await.unwrap();
                });
                false
            }
            Msg::Logout => {
                let kc = self.keycloak.clone();
                spawn_local(async move {
                    // TODO: Handle logout error
                    kc.logout().await.unwrap();
                });
                false
            }
            Msg::UserProfileLoaded(profile) => {
                self.user = Some(profile);
                true
            }
            Msg::ToggleDropdown => {
                self.dropdown_open = !self.dropdown_open;
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        let profile = if self.keycloak.authenticated() {
            let kc = self.keycloak.clone();
            let username = self
                .user
                .as_ref()
                .map(|u| u.username.clone())
                .unwrap_or_else(|| {
                    self.link.send_future(async move {
                        let profile = kc.load_user_profile().await.unwrap();
                        Msg::UserProfileLoaded(profile)
                    });
                    String::from("loading...")
                });

            html! {
                <div>
                    <div class="auth-profile__profile" onclick=self.link.callback(|_| Msg::ToggleDropdown)>
                        <span class="auth-profile__profile__name">{username}</span>
                        <i class="fas fa-chevron-down"/>
                    </div>
                    <Dropdown visible={self.dropdown_open} class=classes!("auth-profile__dropdown")>
                        <Button class=classes!("auth-profile__logout") onclick=self.link.callback(|_| Msg::Logout)>{"Logout"}</Button>
                    </Dropdown>
                </div>
            }
        } else {
            html! {
                <Button onclick=self.link.callback(|_| Msg::Login)>{"Login"}</Button>
            }
        };

        html! {
            <div class="auth-profile">
                {profile}
            </div>
        }
    }
}
