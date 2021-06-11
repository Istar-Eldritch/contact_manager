use yew::prelude::*;

pub struct Button {
    on_click: Callback<MouseEvent>,
    text: String,
    class: Classes,
}

#[derive(Properties, Clone)]
pub struct ButtonProps {
    pub onclick: Callback<MouseEvent>,
    pub text: String,
    #[prop_or_default]
    pub class: Classes,
}

impl Component for Button {
    type Message = ();
    type Properties = ButtonProps;

    fn create(mut props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        props.class.push("button");
        Self {
            on_click: props.onclick,
            text: props.text,
            class: props.class,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.on_click = props.onclick;
        let should_render = self.text.ne(&props.text);
        self.text = props.text;
        self.class = props.class;
        self.class.push("button");
        should_render
    }

    fn view(&self) -> Html {
        html! {
            <button class=self.class.clone() onclick=self.on_click.clone()>{ self.text.clone() }</button>
        }
    }
}
