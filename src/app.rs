use std::collections::HashMap;

use iced::widget::{button, column, row, text_editor};
use iced::{Element, Length};

use crate::layout::{modal, new_task_dialog, swim_lane, view_task_dialog};
use crate::task::{Task, TaskMessage};

pub(crate) const TO_DO: &str = "To do";
pub(crate) const IN_PROGRESS: &str = "In progress";
pub(crate) const DONE: &str = "Done";

#[derive(Debug, Clone)]
pub enum Message {
    TaskMessage(TaskMessage),
    OpenModal(ModalType),
    CloseDialog,
    TaskTitleUpdated(String),
    TaskDescUpdated(text_editor::Action),
    SubmitTask,
}

#[derive(Debug, Clone)]
pub enum ModalType {
    None,
    NewTask,
    ViewTask(u32),
}

pub struct App {
    modal_type: ModalType,
    new_task_text: String,
    new_task_description: text_editor::Content,
    tasks: Vec<Task>,
    next_id: u32,
}

impl App {
    fn hide_dialog(&mut self) {
        self.new_task_text.clear();
        self.new_task_description = text_editor::Content::new();
        self.modal_type = ModalType::None;
    }

    fn find_task_by_id(&self, id: u32) -> Option<&Task> {
        self.tasks.iter().find(|task| task.id == id)
    }

    fn find_task_by_id_mut(&mut self, id: u32) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|task| task.id == id)
    }

    fn remove_task(&mut self, lane: String, task_id: u32) {
        if let Some(pos) = self
            .tasks
            .iter()
            .position(|t| t.lane == lane && t.id == task_id)
        {
            self.tasks.remove(pos);
        }
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::TaskMessage(task_msg) => match task_msg {
                TaskMessage::MoveToLane(_, task_id) => {
                    self.find_task_by_id_mut(task_id)
                        .map(|t| t.update(task_msg));
                }
                TaskMessage::RemoveTask(lane, task_id) => self.remove_task(lane, task_id),
                TaskMessage::OpenModal(modal_type) => self.modal_type = modal_type,
            },
            Message::OpenModal(modal_type) => self.modal_type = modal_type,
            Message::CloseDialog => {
                self.hide_dialog();
            }
            Message::TaskTitleUpdated(task_text) => self.new_task_text = task_text,
            Message::TaskDescUpdated(action) => self.new_task_description.perform(action),
            Message::SubmitTask => {
                let title = self.new_task_text.clone();
                let desc = self.new_task_description.text();
                let task = Task::new(self.next_id, title, desc);
                self.tasks.push(task);
                self.next_id += 1;
                self.hide_dialog();
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let order = [TO_DO, IN_PROGRESS, DONE];
        let mut grouped_by_lane: HashMap<&str, Vec<&Task>> = HashMap::new();

        for task in &self.tasks {
            grouped_by_lane.entry(&task.lane).or_default().push(task);
        }

        let lanes = order.into_iter().map(|lane| {
            let tasks = grouped_by_lane.remove(lane).unwrap_or_default();
            swim_lane(lane.into(), tasks)
        });

        let content = column![
            row![button("Add Task").on_press(Message::OpenModal(ModalType::NewTask))],
            row(lanes).spacing(24)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(4);

        match self.modal_type {
            ModalType::None => content.into(),
            ModalType::NewTask => {
                let add = new_task_dialog(&self.new_task_text, &self.new_task_description);
                modal(content, add, Message::CloseDialog)
            }
            ModalType::ViewTask(task_id) => {
                let task = self.find_task_by_id(task_id);
                if let Some(task) = task {
                    let view = view_task_dialog(task);
                    modal(content, view, Message::CloseDialog)
                } else {
                    // Make an error UI if the ticket id doesn't exist. (Although this shouldn't happen)
                    content.into()
                }
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        let dummy_task_td = Task::new(1, "Test".into(), "This is a test".into());
        let mut dummy_task_ip = Task::new(2, "Test".into(), "This is a test".into());
        dummy_task_ip.lane = IN_PROGRESS.into();

        let tasks = vec![dummy_task_td, dummy_task_ip];

        Self {
            modal_type: ModalType::None,
            new_task_text: String::new(),
            new_task_description: text_editor::Content::new(),
            tasks,
            next_id: 1,
        }
    }
}
