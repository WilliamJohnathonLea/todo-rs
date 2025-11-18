use directories::BaseDirs;
use iced::futures::TryFutureExt;
use iced::widget::{center, text};
use iced::{Element, Subscription, Task, window};
use serde::{Deserialize, Serialize};
use sqlx::migrate::MigrateDatabase;
use sqlx::{Pool, Sqlite, SqlitePool};

use crate::{backlog, projects, sprint, view_controller::ViewController};

const TO_DO: &str = "To do";
const IN_PROGRESS: &str = "In progress";
const DONE: &str = "Done";

const APP_DIR: &str = "todo_rs";
const DB_NAME: &str = "tasks.db";
const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone)]
pub enum Message {
    Initialised(Pool<Sqlite>, Config),
    ProjectsMessage(projects::Message),
    BacklogMessage(backlog::Message),
    TaskMessage(sprint::Message),
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
    db: Pool<Sqlite>,
    current_view: View,
    current_project: Option<projects::Project>,
}

pub enum View {
    Projects(projects::ViewController),
    Backlog(backlog::ViewController),
    Sprint(sprint::ViewController),
}

pub enum App {
    Initiaising,
    Initialised(Initialised),
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self::Initiaising,
            iced::Task::perform(initialise_app(), |res| match res {
                Ok((pool, config)) => Message::Initialised(pool, config),
                Err(err) => panic!("failed to initialise app {err}"),
            }),
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::event::listen().map(|event| Message::EventReceived(event))
    }

    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match self {
            App::Initiaising => self.update_initialising(msg),
            App::Initialised(app) => App::update_initialised(app, msg),
        }
    }

    fn update_initialising(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::Initialised(pool, config) => {
                let (projects_controller, task) = projects::ViewController::new(pool.clone());

                *self = App::Initialised(Initialised {
                    config,
                    db: pool.clone(),
                    current_view: View::Projects(projects_controller),
                    current_project: None,
                });
                task.map(Message::ProjectsMessage)
            }
            Message::EventReceived(iced::Event::Window(iced::window::Event::CloseRequested)) => {
                window::get_latest().and_then(window::close)
            }
            _ => iced::Task::none(),
        }
    }

    fn update_initialised(app: &mut Initialised, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::ProjectsMessage(projects::Message::ProjectOpened(project)) => {
                if let Some(initial_lane) = app.config.lanes.get(0) {
                    // store project in app state and create backlog controller from it
                    let project_for_ctrl = project.clone();
                    let (backlog_controller, task) = backlog::ViewController::new(
                        app.db.clone(),
                        initial_lane.to_owned(),
                        project_for_ctrl,
                    );
                    app.current_view = View::Backlog(backlog_controller);
                    app.current_project = Some(project);
                    task.map(Message::BacklogMessage)
                } else {
                    iced::Task::none()
                }
            }
            Message::ProjectsMessage(projects_msg) => {
                if let View::Projects(controller) = &mut app.current_view {
                    controller
                        .update(projects_msg)
                        .map(Message::ProjectsMessage)
                } else {
                    iced::Task::none()
                }
            }
            Message::TaskMessage(sprint::Message::OpenBacklog) => {
                if let Some(initial_lane) = app.config.lanes.get(0) {
                    if let Some(project) = app.current_project.clone() {
                        let (backlog_controller, task) = backlog::ViewController::new(
                            app.db.clone(),
                            initial_lane.to_owned(),
                            project,
                        );
                        app.current_view = View::Backlog(backlog_controller);
                        task.map(Message::BacklogMessage)
                    } else {
                        iced::Task::none()
                    }
                } else {
                    iced::Task::none()
                }
            }
            Message::TaskMessage(task_msg) => {
                if let View::Sprint(controller) = &mut app.current_view {
                    controller.update(task_msg).map(Message::TaskMessage)
                } else {
                    iced::Task::none()
                }
            }
            Message::BacklogMessage(backlog::Message::OpenSprint) => {
                if let Some(project) = app.current_project.clone() {
                    let (tasks_controller, task) = sprint::ViewController::new(
                        app.db.clone(),
                        app.config.lanes.clone(),
                        project,
                    );
                    app.current_view = View::Sprint(tasks_controller);
                    task.map(Message::TaskMessage)
                } else {
                    iced::Task::none()
                }
            }
            Message::BacklogMessage(backlog_msg) => {
                if let View::Backlog(controller) = &mut app.current_view {
                    controller.update(backlog_msg).map(Message::BacklogMessage)
                } else {
                    iced::Task::none()
                }
            }
            Message::EventReceived(event) => {
                if let iced::Event::Window(iced::window::Event::CloseRequested) = event {
                    iced::Task::future(save_config(app.config.clone()))
                        .and_then(|_| window::get_latest())
                        .and_then(window::close)
                } else {
                    iced::Task::none()
                }
            }
            _ => iced::Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match self {
            App::Initiaising => center(text("Loading...")).into(),
            App::Initialised(app) => match app.current_view {
                View::Projects(ref ctrl) => ctrl.view().map(Message::ProjectsMessage),
                View::Backlog(ref ctrl) => ctrl.view().map(Message::BacklogMessage),
                View::Sprint(ref ctrl) => ctrl.view().map(Message::TaskMessage),
            },
        }
    }
}

async fn setup_app_dirs() -> Result<(), String> {
    let dirs = BaseDirs::new().ok_or("Could not get directories")?;
    let data_dir = dirs.data_dir().join(APP_DIR);
    let conf_dir = dirs.config_dir().join(APP_DIR);

    let data_dir_exists = tokio::fs::try_exists(&data_dir)
        .map_err(|_| format!("Could not check data dir existence"))
        .await?;
    let data = if data_dir_exists {
        Ok(())
    } else {
        tokio::fs::create_dir(&data_dir)
            .map_err(|_| format!("Could not create data dir for app"))
            .await
    };

    let conf_dir_exists = tokio::fs::try_exists(&conf_dir)
        .map_err(|_| format!("Could not check config dir existence"))
        .await?;
    let conf = if conf_dir_exists {
        Ok(())
    } else {
        tokio::fs::create_dir(&conf_dir)
            .map_err(|_| format!("Could not create config dir for app"))
            .await
    };

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
    let conf_file = dirs
        .config_dir()
        .join(format!("{}/{}", APP_DIR, CONFIG_FILE));
    let contents = tokio::fs::read(conf_file)
        .map_err(|err| format!("Unable to load config: {err}"))
        .await?;
    toml::from_slice(&contents).map_err(|err| format!("Unable to parse config: {err}"))
}

async fn save_config(config: Config) -> Result<(), String> {
    let dirs = BaseDirs::new().ok_or("Could not get directories")?;
    let conf_file = dirs
        .config_dir()
        .join(format!("{}/{}", APP_DIR, CONFIG_FILE));
    let serialized = toml::to_string_pretty(&config)
        .map_err(|err| format!("Config serialization error: {}", err))?;
    tokio::fs::write(conf_file, serialized)
        .map_err(|err| format!("Error saving Config: {}", err))
        .await
}

async fn initialise_app() -> Result<(Pool<Sqlite>, Config), String> {
    let pool = setup_app_dirs()
        .and_then(|_| setup_db_connection())
        .and_then(|pool| migrate_db(pool))
        .await?;

    match load_config().await {
        Ok(config) => Ok((pool, config)),
        Err(_) => Ok((pool, Default::default())),
    }
}
