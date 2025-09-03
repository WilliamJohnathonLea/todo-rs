use std::fs;

use directories::BaseDirs;
use iced::widget::{button, column, row};
use iced::{Element, Length, Subscription, window};
use serde::{Deserialize, Serialize};

use crate::layout::modal;
use crate::task;

pub(crate) const TO_DO: &str = "To do";
pub(crate) const IN_PROGRESS: &str = "In progress";
pub(crate) const DONE: &str = "Done";

#[derive(Debug, Clone)]
pub enum Message {
    TaskMessage(task::Message),
    EventReceived(iced::Event),
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
    tasks_controller: task::ViewController,
}

impl App {
    pub fn subscription(&self) -> Subscription<Message> {
        iced::event::listen().map(|event| Message::EventReceived(event))
    }

    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::TaskMessage(task_msg) => {
                self.tasks_controller.update(task_msg);
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
        let base_content =
            || {
                column![
                    row![button("Add Task").on_press(Message::TaskMessage(
                        task::Message::OpenModal(task::Modal::NewTask)
                    ))],
                    self.tasks_controller.view().map(Message::TaskMessage),
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(4)
            };

        self.tasks_controller
            .modal_view()
            .map(|v| {
                modal(
                    base_content(),
                    v.map(Message::TaskMessage),
                    Message::TaskMessage(task::Message::CloseModal),
                )
            })
            .unwrap_or(base_content().into())
    }
}

impl Default for App {
    fn default() -> Self {
        let config = Config::load();
        Self {
            tasks_controller: task::ViewController::new(&config),
            config,
        }
    }
}
