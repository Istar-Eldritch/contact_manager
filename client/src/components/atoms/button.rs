use yew::prelude::*;

pub struct Button {
    props: ButtonProps,
}

#[derive(Properties, Clone)]
pub struct ButtonProps {
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub class: Classes,
    pub children: Children,
    #[prop_or_default]
    pub disabled: bool,
}

impl Component for Button {
    type Message = ();
    type Properties = ButtonProps;

    fn create(mut props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        props.class.push("button");
        Self { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let should_render = self.props.children.ne(&props.children);
        self.props = props;
        self.props.class.push("button");
        should_render
    }

    fn view(&self) -> Html {
        html! {
            <button disabled={self.props.disabled} class=self.props.class.clone() onclick=self.props.onclick.clone()>{ self.props.children.clone() }</button>
        }
    }
}
