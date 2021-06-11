#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<contact_manager_client::Model>();
}
