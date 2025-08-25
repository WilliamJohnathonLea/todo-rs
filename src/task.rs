use iced::widget::{button, column, container, mouse_area, row, text};
use iced::{Element, Length};

use crate::app::ModalType::{self, ViewTask};

#[derive(Default)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub lane: String,
}

#[derive(Clone, Debug)]
pub enum TaskMessage {
    MoveToLane(String, u32),
    RemoveTask(String, u32),
    OpenModal(ModalType),
}

impl Task {
    pub fn new(id: u32, title: String, description: String) -> Self {
        Task {
            id,
            title,
            description,
            lane: crate::app::TO_DO.into(),
        }
    }

    pub fn update(&mut self, msg: TaskMessage) {
        match msg {
            TaskMessage::MoveToLane(new_lane, _) => self.lane = new_lane,
            _ => {}
        }
    }

    pub fn view(&self) -> Element<TaskMessage> {
        let card_content = row![
            column![
                text(&self.title).size(20),
                text(self.id),
                self.move_button(),
            ]
            .width(Length::Fill),
            button("X").on_press(TaskMessage::RemoveTask(self.lane.clone(), self.id))
        ];

        let card = container(card_content)
            .style(container::rounded_box)
            .padding(8)
            .width(Length::Fill);

        mouse_area(card)
            .on_press(TaskMessage::OpenModal(ViewTask(self.id)))
            .into()
    }

    fn move_button(&self) -> Element<TaskMessage> {
        if self.lane == crate::app::TO_DO {
            button(">")
                .on_press(TaskMessage::MoveToLane(
                    crate::app::IN_PROGRESS.into(),
                    self.id,
                ))
                .into()
        } else {
            button(">")
                .on_press(TaskMessage::MoveToLane(crate::app::DONE.into(), self.id))
                .into()
        }
    }
}
