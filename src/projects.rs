use crate::layout::{modal, project_dialog};
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
    OpenNewProject,
    SubmitNewProject,
    ProjectNameUpdated(String),
    SelectProject(i64),
    ProjectOpened(Project),
    OpenEditProject(i64),
    SubmitEditProject(i64),
    DeleteProject(i64),
    ProjectDeleted(i64),
    Cancel,
    NoOp,
}

pub struct ViewController {
    db: Pool<Sqlite>,
    projects: Vec<Project>,
    new_project_name: String,
    modal: Option<Modal>,
    selected_project: Option<i64>,
}

#[derive(Clone, Debug)]
pub enum Modal {
    NewProject,
    EditProject(i64),
}

impl ViewController {
    pub fn new(db: Pool<Sqlite>) -> (Self, iced::Task<Message>) {
        (
            ViewController {
                db: db.clone(),
                projects: vec![],
                new_project_name: Default::default(),
                modal: None,
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
            Message::OpenNewProject => {
                self.modal = Some(Modal::NewProject);
                iced::Task::none()
            }
            Message::OpenEditProject(project_id) => {
                // populate name with current project name
                if let Some(p) = self.projects.iter().find(|p| p.id == project_id) {
                    self.new_project_name = p.name.clone();
                    self.selected_project = Some(project_id);
                }
                self.modal = Some(Modal::EditProject(project_id));
                iced::Task::none()
            }
            Message::SubmitNewProject => {
                let name = self.new_project_name.clone();
                if !name.is_empty() {
                    let db = self.db.clone();
                    self.new_project_name.clear();
                    self.modal = None;
                    iced::Task::perform(insert_project(db.clone(), name), |res| match res {
                        Ok(project) => Message::ProjectOpened(project),
                        Err(_) => Message::NoOp,
                    })
                } else {
                    iced::Task::none()
                }
            }
            Message::DeleteProject(project_id) => {
                let db = self.db.clone();
                iced::Task::perform(delete_project(db.clone(), project_id), move |_| {
                    Message::ProjectDeleted(project_id)
                })
                .chain(iced::Task::perform(
                    get_all_projects(db),
                    Message::ProjectsLoaded,
                ))
            }
            Message::ProjectDeleted(deleted_id) => {
                if self.selected_project == Some(deleted_id) {
                    self.selected_project = None;
                }
                iced::Task::none()
            }
            Message::SubmitEditProject(project_id) => {
                let name = self.new_project_name.clone();
                if !name.is_empty() {
                    let db = self.db.clone();
                    self.new_project_name.clear();
                    self.modal = None;
                    iced::Task::perform(update_project(db.clone(), project_id, name), |_| {
                        Message::NoOp
                    })
                    .chain(iced::Task::perform(
                        get_all_projects(db),
                        Message::ProjectsLoaded,
                    ))
                } else {
                    iced::Task::none()
                }
            }
            Message::Cancel => {
                self.modal = None;
                self.new_project_name.clear();
                iced::Task::none()
            }
            Message::SelectProject(project_id) => {
                self.selected_project = Some(project_id);
                iced::Task::none()
            }
            Message::ProjectOpened(_project) => iced::Task::none(), // Handled at the App level
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
                let name_btn = button(text(&project.name).width(Length::Fill).size(14))
                    .width(Length::Fill)
                    .padding(12)
                    .on_press(Message::SelectProject(project.id))
                    .style(move |theme, status| {
                        if is_selected {
                            iced::widget::button::primary(theme, status)
                        } else {
                            iced::widget::button::secondary(theme, status)
                        }
                    });

                let edit_btn = button(text("✎").size(14))
                    .padding(8)
                    .on_press(Message::OpenEditProject(project.id));

                let delete_btn = button(text("X").size(14))
                    .padding(8)
                    .on_press(Message::DeleteProject(project.id));

                let row = row![name_btn, edit_btn, delete_btn]
                    .spacing(8)
                    .width(Length::Fill);

                list = list.push(row);
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
            .on_press(Message::OpenNewProject)
            .padding(10);

        let open_button = if let Some(selected_id) = self.selected_project {
            // find the selected project and capture it for the message
            if let Some(p) = self.projects.iter().find(|p| p.id == selected_id) {
                button(text("Open Project").size(14))
                    .on_press(Message::ProjectOpened(p.clone()))
                    .padding(10)
                    .style(iced::widget::button::success)
            } else {
                button(text("Open Project").size(14))
                    .padding(10)
                    .style(iced::widget::button::secondary)
            }
        } else {
            button(text("Open Project").size(14))
                .padding(10)
                .style(iced::widget::button::secondary)
        };

        let button_row = row![create_button, open_button].spacing(8).padding(16);

        let base = column![projects_section, button_row].spacing(8).padding(16);

        match &self.modal {
            Some(Modal::EditProject(project_id)) => {
                let dialog = project_dialog(
                    "Edit Project".into(),
                    &self.new_project_name,
                    &Message::ProjectNameUpdated,
                    Message::SubmitEditProject(*project_id),
                    Message::Cancel,
                );

                modal(base, dialog, Message::Cancel)
            }
            Some(Modal::NewProject) => {
                let dialog = project_dialog(
                    "New Project".into(),
                    &self.new_project_name,
                    &Message::ProjectNameUpdated,
                    Message::SubmitNewProject,
                    Message::Cancel,
                );

                modal(base, dialog, Message::Cancel)
            }
            None => base.into(),
        }
    }
}

pub async fn get_all_projects(pool: Pool<Sqlite>) -> Result<Vec<Project>, String> {
    sqlx::query_as!(Project, "SELECT id, name FROM projects ORDER BY id")
        .fetch_all(&pool)
        .map_err(|err| format!("Error loading projects: {err}"))
        .await
}

pub async fn insert_project(pool: Pool<Sqlite>, name: String) -> Result<Project, String> {
    let res = sqlx::query!("INSERT INTO projects (name) VALUES (?)", name)
        .execute(&pool)
        .await
        .map_err(|_| String::from("Error inserting project into db"))?;

    let id = res.last_insert_rowid();
    Ok(Project { id, name })
}

pub async fn delete_project(pool: Pool<Sqlite>, project_id: i64) -> Result<(), String> {
    // delete tasks for project then delete project
    sqlx::query!("DELETE FROM tasks WHERE project_id = ?", project_id)
        .execute(&pool)
        .await
        .map_err(|_| "Error deleting tasks for project".to_string())?;

    sqlx::query!("DELETE FROM projects WHERE id = ?", project_id)
        .execute(&pool)
        .await
        .map_err(|_| "Error deleting project".to_string())?;

    Ok(())
}

pub async fn update_project(
    pool: Pool<Sqlite>,
    project_id: i64,
    name: String,
) -> Result<(), String> {
    sqlx::query!(
        "UPDATE projects SET name = ? WHERE id = ?",
        name,
        project_id
    )
    .execute(&pool)
    .await
    .map_err(|_| "Error updating project".to_string())?;

    Ok(())
}
