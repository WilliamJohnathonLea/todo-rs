use std::fs;

use directories::BaseDirs;
use iced::widget::{button, column, row};
use iced::{Element, Length, Subscription, window};
use serde::{Deserialize, Serialize};

use crate::task;

pub(crate) const TO_DO: &str = "To do";
pub(crate) const IN_PROGRESS: &str = "In progress";
pub(crate) const DONE: &str = "Done";

#[derive(Debug, Clone)]
pub enum Message {
    TaskMessage(task::Message),
    OpenModal(ModalType),
    CloseDialog,
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
    pub lanes: Vec<String>,
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
    tasks_controller: task::ViewController,
}

impl App {
    fn hide_dialog(&mut self) {
        self.modal_type = ModalType::None;
    }

    // fn find_task_by_id(&self, id: u32) -> Option<&Task> {
    //     self.tasks.iter().find(|task| task.id == id)
    // }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::event::listen().map(|event| Message::EventReceived(event))
    }

    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::TaskMessage(task_msg) => {
                self.tasks_controller.update(task_msg);
                iced::Task::none()
            }
            Message::OpenModal(modal_type) => {
                self.modal_type = modal_type;
                iced::Task::none()
            }
            Message::CloseDialog => {
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
        let content = column![
            row![button("Add Task").on_press(Message::OpenModal(ModalType::NewTask))],
            self.tasks_controller.view().map(Message::TaskMessage),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(4);

        // match self.modal_type {
        //     ModalType::None => content.into(),
        //     ModalType::NewTask => {
        //         let add = new_task_dialog(&self.new_task_text, &self.new_task_description);
        //         modal(content, add, Message::CloseDialog)
        //     }
        //     ModalType::ViewTask(task_id) => {
        //         let task = self.find_task_by_id(task_id);
        //         if let Some(task) = task {
        //             let view = view_task_dialog(task);
        //             modal(content, view, Message::CloseDialog)
        //         } else {
        //             // Make an error UI if the ticket id doesn't exist. (Although this shouldn't happen)
        //             content.into()
        //         }
        //     }
        // }
        content.into()
    }
}

impl Default for App {
    fn default() -> Self {
        let config = Config::load();
        Self {
            tasks_controller: task::ViewController::new(&config),
            config,
            modal_type: ModalType::None,
        }
    }
}
