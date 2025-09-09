use directories::BaseDirs;
use iced::futures::TryFutureExt;
use iced::widget::{button, center, column, row, text};
use iced::{Element, Length, Subscription, Task, window};
use serde::{Deserialize, Serialize};
use sqlx::migrate::MigrateDatabase;
use sqlx::{Pool, Sqlite, SqlitePool};

use crate::layout::modal;
use crate::task;

const TO_DO: &str = "To do";
const IN_PROGRESS: &str = "In progress";
const DONE: &str = "Done";

const APP_DIR: &str = "todo_rs";
const DB_NAME: &str = "tasks.db";

#[derive(Debug, Clone)]
pub enum Message {
    NoOp,
    ConfigLoaded(Result<Config, String>),
    DbConnected(Result<Pool<Sqlite>, String>),
    DbMigrated(Result<Pool<Sqlite>, String>),
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

pub struct Initialised {
    config: Config,
    tasks_controller: task::ViewController,
}

pub enum App {
    Initiaising,
    Initialised(Initialised),
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self::Initiaising,
            iced::Task::batch([
                iced::Task::perform(setup_app_dirs(), |_| Message::NoOp),
                // iced::Task::perform(setup_db_connection(), Message::DbConnected),
                iced::Task::perform(load_config(), Message::ConfigLoaded),
            ]),
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::event::listen().map(|event| Message::EventReceived(event))
    }

    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match self {
            App::Initiaising => iced::Task::none(),
            App::Initialised(app) => App::update_initialised(app, msg),
        }
    }

    fn update_initialised(app: &mut Initialised, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::ConfigLoaded(config_result) => {
                if let Ok(config) = config_result {
                    app.config = config
                }
                app.tasks_controller.configure(&app.config);
                iced::Task::none()
            }
            Message::DbConnected(db) => {
                if let Ok(db) = db {
                    iced::Task::perform(migrate_db(db), Message::DbMigrated)
                } else {
                    iced::Task::none()
                }
            }
            Message::DbMigrated(db) => {
                if let Ok(db) = db {
                    iced::Task::perform(task::get_tasks(db), |res| {
                        Message::TaskMessage(task::Message::TasksLoaded(res))
                    })
                } else {
                    iced::Task::none()
                }
            }
            Message::TaskMessage(task_msg) => app
                .tasks_controller
                .update(task_msg)
                .map(Message::TaskMessage),
            Message::EventReceived(event) => {
                if let iced::Event::Window(iced::window::Event::CloseRequested) = event {
                    iced::Task::future(save_config(app.config.clone()))
                        .and_then(|_| window::get_latest())
                        .and_then(window::close)
                } else {
                    iced::Task::none()
                }
            }
            Message::NoOp => iced::Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        match self {
            App::Initiaising => center(text("Loading...")).into(),
            App::Initialised(app) => {
                let base_content = || {
                    column![
                        row![button("Add Task").on_press(Message::TaskMessage(
                            task::Message::OpenModal(task::Modal::NewTask)
                        ))],
                        app.tasks_controller.view().map(Message::TaskMessage),
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .spacing(4)
                };

                app.tasks_controller
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
    }
}

async fn setup_app_dirs() -> Result<(), String> {
    let dirs = BaseDirs::new().ok_or("Could not get directories")?;
    let data_dir = dirs.data_dir().join(APP_DIR);
    let conf_dir = dirs.config_dir().join(APP_DIR);

    let data = tokio::fs::create_dir(data_dir)
        .map_err(|_| format!("Could not create data dir for app"))
        .await;
    let conf = tokio::fs::create_dir(conf_dir)
        .map_err(|_| format!("Could not create config dir for app"))
        .await;

    data.and_then(|_| conf)
}

async fn setup_db_connection() -> Result<Pool<Sqlite>, String> {
    let dirs = BaseDirs::new().ok_or("Could not get directories")?;
    let data_dir = dirs.data_dir().join(APP_DIR);
    let db_file = data_dir.join(DB_NAME);
    let db_url = db_file
        .to_str()
        .map(|s| format!("sqlite://{}", s))
        .ok_or("Could not create valid db url")?;

    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        match Sqlite::create_database(&db_url).await {
            Ok(_) => {
                SqlitePool::connect(&db_url)
                    .map_err(|_| "Could not connect to db".into())
                    .await
            }
            Err(_) => Err("error creating db".into()),
        }
    } else {
        SqlitePool::connect(&db_url)
            .map_err(|_| "Could not connect to db".into())
            .await
    }
}

async fn migrate_db(pool: Pool<Sqlite>) -> Result<Pool<Sqlite>, String> {
    sqlx::migrate!()
        .run(&pool)
        .map_err(|_| String::from("failed to run migration"))
        .await?;
    Ok(pool)
}

async fn load_config() -> Result<Config, String> {
    let dirs = BaseDirs::new().ok_or("Could not get directories")?;
    let conf_file = dirs.config_dir().join("todo_rs/config.toml");
    let contents = tokio::fs::read(conf_file)
        .map_err(|err| err.to_string())
        .await?;
    toml::from_slice(&contents).map_err(|err| err.to_string())
}

async fn save_config(config: Config) -> Result<(), String> {
    let dirs = BaseDirs::new().ok_or("Could not get directories")?;
    let conf_file = dirs.config_dir().join("todo_rs/config.toml");
    let serialized = toml::to_string_pretty(&config)
        .map_err(|err| format!("Config serialization error: {}", err))?;
    tokio::fs::write(conf_file, serialized)
        .map_err(|err| format!("Error saving Config: {}", err))
        .await
}
