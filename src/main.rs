mod app;
mod dsl;
mod home;
mod registry;
mod sync;
mod tools;
mod ui;

use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(app::App);
}
