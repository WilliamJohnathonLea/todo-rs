use crate::app::App;

mod app;
mod layout;
mod task;

fn main() -> iced::Result {
    iced::application("ToDo", App::update, App::view)
        .subscription(App::subscription)
        .theme(|_| iced::Theme::KanagawaDragon)
        .exit_on_close_request(false)
        .run_with(App::new)
}
