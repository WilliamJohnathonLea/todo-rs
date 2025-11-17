use crate::view_controller::ViewController as VC;
use iced::futures::TryFutureExt;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};
use sqlx::{Pool, Sqlite};

#[derive(Clone, Debug)]
pub struct Project {
    pub id: i64,
    pub name: String,
}

#[derive(Clone, Debug)]
pub enum Message {
    ProjectsLoaded(Result<Vec<Project>, String>),
    CreateProject,
    ProjectNameUpdated(String),
    OpenProject(i64),
    NoOp,
}

pub struct ViewController {
    db: Pool<Sqlite>,
    projects: Vec<Project>,
    new_project_name: String,
    selected_project: Option<i64>,
}

impl ViewController {
    pub fn new(db: Pool<Sqlite>) -> (Self, iced::Task<Message>) {
        (
            ViewController {
                db: db.clone(),
                projects: vec![],
                new_project_name: Default::default(),
                selected_project: None,
            },
            iced::Task::perform(get_all_projects(db), Message::ProjectsLoaded),
        )
    }
}

impl VC for ViewController {
    type Message = Message;

    fn update(&mut self, msg: Self::Message) -> iced::Task<Self::Message> {
        match msg {
            Message::ProjectsLoaded(result) => {
                if let Ok(projects) = result {
                    self.projects = projects;
                }
                iced::Task::none()
            }
            Message::ProjectNameUpdated(name) => {
                self.new_project_name = name;
                iced::Task::none()
            }
            Message::CreateProject => {
                let name = self.new_project_name.clone();
                if !name.is_empty() {
                    let db = self.db.clone();
                    self.new_project_name.clear();
                    iced::Task::perform(insert_project(db.clone(), name), |_| Message::NoOp).chain(
                        iced::Task::perform(get_all_projects(db), Message::ProjectsLoaded),
                    )
                } else {
                    iced::Task::none()
                }
            }
            Message::OpenProject(project_id) => {
                self.selected_project = Some(project_id);
                iced::Task::none()
            }
            Message::NoOp => iced::Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let title = text("Projects").size(32);
        let subtitle = text("Select a project...").size(16);

        let projects_list = if self.projects.is_empty() {
            column![text("No projects yet").size(14)].spacing(8)
        } else {
            let mut list = column![].spacing(4);
            for project in &self.projects {
                let is_selected = self.selected_project == Some(project.id);
                let project_button = button(text(&project.name).width(Length::Fill).size(14))
                    .width(Length::Fill)
                    .padding(12)
                    .on_press(Message::OpenProject(project.id))
                    .style(move |theme, status| {
                        if is_selected {
                            iced::widget::button::primary(theme, status)
                        } else {
                            iced::widget::button::secondary(theme, status)
                        }
                    });

                list = list.push(project_button);
            }
            list
        };

        let projects_section = column![
            title,
            subtitle,
            container(projects_list)
                .padding(16)
                .width(Length::Fill)
                .height(Length::FillPortion(4))
        ]
        .spacing(8);

        let create_button = button(text("+ Create Project").size(14))
            .on_press(Message::CreateProject)
            .padding(10);

        let open_button = if self.selected_project.is_some() {
            button(text("Open Project").size(14))
                .on_press(Message::OpenProject(self.selected_project.unwrap()))
                .padding(10)
                .style(iced::widget::button::success)
        } else {
            button(text("Open Project").size(14))
                .padding(10)
                .style(iced::widget::button::secondary)
        };

        let button_row = row![create_button, open_button].spacing(8).padding(16);

        column![projects_section, button_row]
            .spacing(8)
            .padding(16)
            .into()
    }
}

pub async fn get_all_projects(pool: Pool<Sqlite>) -> Result<Vec<Project>, String> {
    sqlx::query_as!(Project, "SELECT id, name FROM projects ORDER BY id")
        .fetch_all(&pool)
        .map_err(|err| format!("Error loading projects: {err}"))
        .await
}

pub async fn insert_project(pool: Pool<Sqlite>, name: String) -> Result<(), String> {
    sqlx::query!("INSERT INTO projects (name) VALUES (?)", name)
        .execute(&pool)
        .map_err(|_| "Error inserting project into db".into())
        .map_ok(|_| ())
        .await
}
