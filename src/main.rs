use crate::app::App;

mod app;
mod backlog;
mod layout;
mod task;
mod view_controller;

fn main() -> iced::Result {
    iced::application("ToDo", App::update, App::view)
        .subscription(App::subscription)
        .theme(|_| iced::Theme::KanagawaDragon)
        .centered()
        .exit_on_close_request(false)
        .run_with(App::new)
}
