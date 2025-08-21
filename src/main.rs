use crate::app::App;

mod app;
mod layout;
mod task;

fn main() -> iced::Result {
    iced::application("ToDo", App::update, App::view)
        .theme(|_| iced::Theme::KanagawaDragon)
        .run()
}
