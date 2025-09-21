use crate::layout;
use crate::view_controller::ViewController as VC;

use iced::Element;

#[derive(Clone, Debug)]
pub enum Message {}

pub struct ViewController {}

impl ViewController {
    pub fn new() -> Self {
        ViewController {}
    }
}

impl VC for ViewController {
    type Message = Message;

    fn update(&mut self, _msg: Self::Message) -> iced::Task<Self::Message> {
        todo!()
    }

    fn view(&self) -> iced::Element<Self::Message> {
        todo!()
    }
}
