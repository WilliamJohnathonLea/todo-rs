use std::collections::HashMap;
use std::vec;

use iced::futures::TryFutureExt;
use iced::widget::{button, column, row, text_editor};
use iced::{Element, Length};
use sqlx::{Pool, Sqlite};

use crate::layout::{modal, swim_lane, task_card, task_dialog, task_dialog_mut};
use crate::view_controller::ViewController as VC;

#[derive(Clone, Debug, Default)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub lane: String,
}

#[derive(Clone, Debug, Default)]
struct NewTask {
    title: String,
    description: Option<String>,
    lane: String,
}

#[derive(Clone, Debug)]
pub enum Message {
    TasksLoaded(Result<Vec<Task>, String>),
    CreateTask,
    EditTask(i64),
    RemoveTask(i64),
    MoveToLane(String, i64),
    OpenModal(Modal),
    CloseModal,
    TaskTitleUpdated(String),
    TaskDescUpdated(text_editor::Action),
    OpenBacklog,
    NoOp,
}

#[derive(Clone, Debug)]
pub enum Modal {
    NewTask,
    ViewTask(i64),
    EditTask(i64),
}

pub struct ViewController {
    modal: Option<Modal>,
    db: Pool<Sqlite>,
    lanes: Vec<String>,
    tasks: Vec<Task>,
    new_task_title: String,
    new_task_description: text_editor::Content,
}

impl NewTask {
    pub fn new(title: String, description: Option<String>, lane: String) -> Self {
        NewTask {
            title,
            description,
            lane,
        }
    }
}

impl ViewController {
    pub fn new(db: Pool<Sqlite>, lanes: Vec<String>) -> Self {
        Self {
            modal: None,
            db,
            lanes,
            tasks: vec![],
            new_task_title: Default::default(),
            new_task_description: Default::default(),
        }
    }

    fn hide_dialog(&mut self) {
        self.new_task_title.clear();
        self.new_task_description = text_editor::Content::new();
        self.modal = None;
    }

    fn find_task_by_id(&self, id: i64) -> Option<&Task> {
        self.tasks.iter().find(|task| task.id == id)
    }

    fn find_task_by_id_mut(&mut self, id: i64) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|task| task.id == id)
    }

    fn modal_view(&self) -> Option<Element<Message>> {
        match self.modal {
            Some(Modal::ViewTask(task_id)) => {
                let maybe_task = self.find_task_by_id(task_id);
                maybe_task.map(|t| {
                    task_dialog(
                        t,
                        Message::OpenModal(Modal::EditTask(t.id)),
                        Message::CloseModal,
                    )
                })
            }
            Some(Modal::NewTask) => Some(task_dialog_mut(
                "New Task".into(),
                &self.new_task_title,
                &self.new_task_description,
                &Message::TaskTitleUpdated,
                &Message::TaskDescUpdated,
                Message::CreateTask,
                Message::CloseModal,
            )),
            Some(Modal::EditTask(task_id)) => Some(task_dialog_mut(
                "Edit Task".into(),
                &self.new_task_title,
                &self.new_task_description,
                &Message::TaskTitleUpdated,
                &Message::TaskDescUpdated,
                Message::EditTask(task_id),
                Message::CloseModal,
            )),
            None => None,
        }
    }
}

impl VC for ViewController {
    type Message = Message;

    fn update(&mut self, msg: Self::Message) -> iced::Task<Self::Message> {
        match msg {
            Message::TasksLoaded(tasks) => {
                if let Ok(tasks) = tasks {
                    self.tasks = tasks
                }
                iced::Task::none()
            }
            Message::CreateTask => {
                let title = self.new_task_title.clone();
                let desc = Some(self.new_task_description.text());
                if let Some(lane) = self.lanes.get(0) {
                    let task = NewTask::new(title, desc, lane.clone());
                    iced::Task::perform(insert_task(self.db.clone(), task), |_| Message::CloseModal)
                        .chain(iced::Task::perform(
                            get_tasks(self.db.clone()),
                            Message::TasksLoaded,
                        ))
                } else {
                    iced::Task::none()
                }
            }
            Message::EditTask(task_id) => {
                let title = self.new_task_title.clone();
                let desc = self.new_task_description.text();
                let db = self.db.clone();
                if let Some(task) = self.find_task_by_id_mut(task_id) {
                    task.title = title;
                    task.description = Some(desc);
                    iced::Task::perform(edit_task(db, task.clone()), |_| Message::CloseModal)
                } else {
                    iced::Task::done(Message::CloseModal)
                }
            }
            Message::RemoveTask(task_id) => {
                iced::Task::perform(remove_task(self.db.clone(), task_id), |_| Message::NoOp).chain(
                    iced::Task::perform(get_tasks(self.db.clone()), Message::TasksLoaded),
                )
            }
            Message::MoveToLane(new_lane, task_id) => {
                let db = self.db.clone();
                if let Some(task) = self.find_task_by_id_mut(task_id) {
                    task.lane = new_lane;
                    iced::Task::perform(edit_task(db, task.clone()), |_| Message::NoOp)
                } else {
                    iced::Task::none()
                }
            }
            Message::OpenModal(modal) => {
                if let Modal::EditTask(task_id) = modal {
                    if let Some(task) = self.find_task_by_id(task_id) {
                        let desc = task.description.clone();
                        self.new_task_title = task.title.clone();
                        if let Some(desc) = desc {
                            self.new_task_description = text_editor::Content::with_text(&desc);
                        }
                    }
                }
                self.modal = Some(modal);
                iced::Task::none()
            }
            Message::CloseModal => {
                self.hide_dialog();
                iced::Task::none()
            }
            Message::TaskTitleUpdated(task_text) => {
                self.new_task_title = task_text;
                iced::Task::none()
            }
            Message::TaskDescUpdated(action) => {
                self.new_task_description.perform(action);
                iced::Task::none()
            }
            Message::OpenBacklog => iced::Task::none(), // Handled at the App level
            Message::NoOp => iced::Task::none(),
        }
    }

    fn view(&self) -> iced::Element<Self::Message> {
        let mut grouped_by_lane: HashMap<&str, Vec<&Task>> = HashMap::new();

        for task in &self.tasks {
            grouped_by_lane.entry(&task.lane).or_default().push(task);
        }

        let lanes = self.lanes.iter().enumerate().map(|(idx, lane)| {
            let tasks = grouped_by_lane.remove(lane.as_str()).unwrap_or_default();
            let elems = tasks
                .iter()
                .map(|t| {
                    let next_lane = self
                        .lanes
                        .get(idx + 1)
                        .map(|lane| Message::MoveToLane(lane.clone(), t.id));
                    task_card(
                        t,
                        Message::RemoveTask(t.id),
                        Message::OpenModal(Modal::ViewTask(t.id)),
                        next_lane,
                    )
                })
                .collect();

            let title = format!("{} ({})", lane, tasks.len());
            swim_lane(title, elems)
        });

        let base_content = column![
            row![
                button("Backlog").on_press(Message::OpenBacklog),
                button("Add Task").on_press(Message::OpenModal(Modal::NewTask))
            ]
            .spacing(4),
            row(lanes).spacing(24),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(4);

        if let Some(v) = self.modal_view() {
            modal(base_content, v, Message::CloseModal)
        } else {
            base_content.into()
        }
    }
}

pub async fn get_tasks(pool: Pool<Sqlite>) -> Result<Vec<Task>, String> {
    sqlx::query_as!(Task, "SELECT id, title, description, lane FROM tasks")
        .fetch_all(&pool)
        .map_err(|err| format!("got db err: {err}"))
        .await
}

async fn insert_task(pool: Pool<Sqlite>, t: NewTask) -> Result<(), String> {
    sqlx::query!(
        "INSERT INTO tasks (title, description, lane) VALUES (?, ?, ?)",
        t.title,
        t.description,
        t.lane
    )
    .execute(&pool)
    .map_err(|_| "Error inserting task into db".into())
    .map_ok(|_| ())
    .await
}

async fn remove_task(pool: Pool<Sqlite>, task_id: i64) -> Result<(), String> {
    sqlx::query!("DELETE FROM tasks WHERE id = ?", task_id)
        .execute(&pool)
        .map_err(|_| "Error deleting task from db".into())
        .map_ok(|_| ())
        .await
}

async fn edit_task(pool: Pool<Sqlite>, task: Task) -> Result<(), String> {
    sqlx::query!(
        "UPDATE tasks SET title = ?, description = ?, lane = ? WHERE id = ?",
        task.title,
        task.description,
        task.lane,
        task.id
    )
    .execute(&pool)
    .map_err(|_| "Error deleting task from db".into())
    .map_ok(|_| ())
    .await
}
