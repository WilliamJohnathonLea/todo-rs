use std::collections::HashMap;
use std::fs;

use directories::BaseDirs;
use iced::widget::{button, column, row, text_editor};
use iced::{Element, Length, Subscription, window};
use serde::{Deserialize, Serialize};

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
    EventReceived(iced::Event),
}

#[derive(Debug, Clone)]
pub enum ModalType {
    None,
    NewTask,
    ViewTask(u32),
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    lanes: Vec<String>,
}

impl Config {
    pub fn load() -> Config {
        if let Some(base_dirs) = BaseDirs::new() {
            let path = base_dirs.config_dir().join("todo_rs.toml");
            fs::read(path)
                .map_err(|err| err.to_string())
                .and_then(|contents| toml::from_slice(&contents).map_err(|err| err.to_string()))
                .unwrap_or_else(|err| {
                    println!("Error loading config file: {}", err);
                    Default::default()
                })
        } else {
            Default::default()
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let dirs = BaseDirs::new().ok_or("Could not get directories")?;
        let conf_file = dirs.config_dir().join("todo_rs.toml");
        let serialized = toml::to_string_pretty(self)
            .map_err(|err| format!("Config serialization error: {}", err))?;
        fs::write(conf_file, serialized).map_err(|err| format!("Error saving Config: {}", err))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lanes: vec![TO_DO.into(), IN_PROGRESS.into(), DONE.into()],
        }
    }
}

pub struct App {
    config: Config,
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

    pub fn subscription(&self) -> Subscription<Message> {
        iced::event::listen().map(|event| Message::EventReceived(event))
    }

    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::TaskMessage(task_msg) => match task_msg {
                TaskMessage::MoveToLane(_, task_id) => {
                    self.find_task_by_id_mut(task_id)
                        .map(|t| t.update(task_msg));
                    iced::Task::none()
                }
                TaskMessage::RemoveTask(lane, task_id) => {
                    self.remove_task(lane, task_id);
                    iced::Task::none()
                }
                TaskMessage::OpenModal(modal_type) => {
                    self.modal_type = modal_type;
                    iced::Task::none()
                }
            },
            Message::OpenModal(modal_type) => {
                self.modal_type = modal_type;
                iced::Task::none()
            }
            Message::CloseDialog => {
                self.hide_dialog();
                iced::Task::none()
            }
            Message::TaskTitleUpdated(task_text) => {
                self.new_task_text = task_text;
                iced::Task::none()
            }
            Message::TaskDescUpdated(action) => {
                self.new_task_description.perform(action);
                iced::Task::none()
            }
            Message::SubmitTask => {
                let title = self.new_task_text.clone();
                let desc = self.new_task_description.text();
                let task = Task::new(self.next_id, title, desc);
                self.tasks.push(task);
                self.next_id += 1;
                self.hide_dialog();
                iced::Task::none()
            }
            Message::EventReceived(event) => {
                if let iced::Event::Window(iced::window::Event::CloseRequested) = event {
                    let _ = self.config.save();
                    window::get_latest().and_then(window::close)
                } else {
                    iced::Task::none()
                }
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let order = &self.config.lanes;
        let mut grouped_by_lane: HashMap<&str, Vec<&Task>> = HashMap::new();

        for task in &self.tasks {
            grouped_by_lane.entry(&task.lane).or_default().push(task);
        }

        let lanes = order.into_iter().map(|lane| {
            let tasks = grouped_by_lane.remove(lane.as_str()).unwrap_or_default();
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
        let config = Config::load();
        Self {
            config,
            modal_type: ModalType::None,
            new_task_text: String::new(),
            new_task_description: text_editor::Content::new(),
            tasks: vec![],
            next_id: 1,
        }
    }
}
