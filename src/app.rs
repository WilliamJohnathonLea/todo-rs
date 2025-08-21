use std::collections::HashMap;

use iced::widget::{button, column, row, text_editor};
use iced::{Element, Length};

use crate::layout::{modal, new_task_dialog, swim_lane, view_task_dialog};
use crate::task::Task;

const TO_DO: &str = "To do";
const IN_PROGRESS: &str = "In progress";
const DONE: &str = "Done";

#[derive(Debug, Clone)]
pub enum Message {
    OpenDialog(ModalType),
    CloseDialog,
    TaskTitleUpdated(String),
    TaskDescUpdated(text_editor::Action),
    SubmitTask,
    RemoveTask,
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
    tasks: HashMap<String, Vec<Task>>,
    next_id: u32,
}

impl App {
    fn hide_dialog(&mut self) {
        self.new_task_text.clear();
        self.new_task_description = text_editor::Content::new();
        self.modal_type = ModalType::None;
    }

    fn find_task_by_id(&self, id: u32) -> Option<&Task> {
        for tasks_vec in self.tasks.values() {
            if let Some(task) = tasks_vec.iter().find(|task| task.id == id) {
                return Some(task);
            }
        }
        None
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::OpenDialog(modal_type) => self.modal_type = modal_type,
            Message::CloseDialog => {
                self.hide_dialog();
            }
            Message::TaskTitleUpdated(task_text) => self.new_task_text = task_text,
            Message::TaskDescUpdated(action) => self.new_task_description.perform(action),
            Message::SubmitTask => {
                let title = self.new_task_text.clone();
                let desc = self.new_task_description.text();
                let task = Task::new(self.next_id, title, desc);
                self.tasks.entry(TO_DO.into()).or_default().push(task);
                self.next_id += 1;
                self.hide_dialog();
            }
            Message::RemoveTask => println!("Task removed!"),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let order = [TO_DO, IN_PROGRESS, DONE];
        let lanes = order
            .into_iter()
            .filter_map(|key| self.tasks.get(key).map(|v| (key, v)))
            .map(|(k, v)| swim_lane(k, v));

        let content = column![
            row![button("Add Task").on_press(Message::OpenDialog(ModalType::NewTask))],
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
        let mut tasks = HashMap::new();
        tasks.insert(TO_DO.into(), vec![]);
        tasks.insert(IN_PROGRESS.into(), vec![]);
        tasks.insert(DONE.into(), vec![]);

        Self {
            modal_type: ModalType::None,
            new_task_text: String::new(),
            new_task_description: text_editor::Content::new(),
            tasks,
            next_id: 1,
        }
    }
}
