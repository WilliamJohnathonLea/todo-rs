pub trait ViewController {
    type Message;

    fn update(&mut self, msg: Self::Message) -> iced::Task<Self::Message>;

    fn view(&self) -> iced::Element<Self::Message>;
}
