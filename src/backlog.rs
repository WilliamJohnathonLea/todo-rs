use crate::layout;
use crate::view_controller::ViewController as VC;

use iced::Length;
use iced::widget::{button, column, text};

#[derive(Clone, Debug)]
pub enum Message {
    OpenSprint,
}

pub struct ViewController {}

impl ViewController {
    pub fn new() -> Self {
        ViewController {}
    }
}

impl VC for ViewController {
    type Message = Message;

    fn update(&mut self, msg: Self::Message) -> iced::Task<Self::Message> {
        match msg {
            Message::OpenSprint => iced::Task::none(), // Handled at the App level
        }
    }

    fn view(&self) -> iced::Element<Self::Message> {
        column![
            button("Sprint").on_press(Message::OpenSprint),
            text("Backlog").size(24),
            layout::backlog(vec![])
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(4)
        .into()
    }
}
