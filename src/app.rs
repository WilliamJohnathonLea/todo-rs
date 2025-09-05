use directories::BaseDirs;
use iced::futures::TryFutureExt;
use iced::widget::{button, column, row};
use iced::{Element, Length, Subscription, Task, window};
use serde::{Deserialize, Serialize};

use crate::layout::modal;
use crate::task;

const TO_DO: &str = "To do";
const IN_PROGRESS: &str = "In progress";
const DONE: &str = "Done";

#[derive(Debug, Clone)]
pub enum Message {
    ConfigLoaded(Result<Config, String>),
    TaskMessage(task::Message),
    EventReceived(iced::Event),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub lanes: Vec<String>,
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
    pub fn new() -> (Self, Task<Message>) {
        (
            Default::default(),
            iced::Task::perform(load_config(), Message::ConfigLoaded),
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::event::listen().map(|event| Message::EventReceived(event))
    }

    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::ConfigLoaded(config) => {
                if let Ok(config) = config {
                    self.config = config
                }
                self.tasks_controller.configure(&self.config);
                iced::Task::none()
            }
            Message::TaskMessage(task_msg) => {
                self.tasks_controller.update(task_msg);
                iced::Task::none()
            }
            Message::EventReceived(event) => {
                if let iced::Event::Window(iced::window::Event::CloseRequested) = event {
                    iced::Task::future(save_config(self.config.clone()))
                        .and_then(|_| window::get_latest())
                        .and_then(window::close)
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
        Self {
            config: Default::default(),
            tasks_controller: task::ViewController::new(),
        }
    }
}

async fn load_config() -> Result<Config, String> {
    let dirs = BaseDirs::new().ok_or("Could not get directories")?;
    let conf_file = dirs.config_dir().join("todo_rs.toml");
    let contents = tokio::fs::read(conf_file)
        .map_err(|err| err.to_string())
        .await?;
    toml::from_slice(&contents).map_err(|err| err.to_string())
}

async fn save_config(config: Config) -> Result<(), String> {
    let dirs = BaseDirs::new().ok_or("Could not get directories")?;
    let conf_file = dirs.config_dir().join("todo_rs.toml");
    let serialized = toml::to_string_pretty(&config)
        .map_err(|err| format!("Config serialization error: {}", err))?;
    tokio::fs::write(conf_file, serialized)
        .map_err(|err| format!("Error saving Config: {}", err))
        .await
}
