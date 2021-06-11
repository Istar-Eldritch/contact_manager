use yew::prelude::*;

pub struct Dropdown {
    props: Props,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub children: Children,
    #[prop_or_default]
    pub class: Classes,
    pub visible: bool,
}

impl Component for Dropdown {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let mut classes = self.props.class.clone();
        classes.push("dropdown");
        if !self.props.visible {
            classes.push("dropdown--hidden");
        }
        html! {
            <div class=classes>{self.props.children.clone()}</div>
        }
    }
}
