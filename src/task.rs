use std::collections::HashMap;
use std::vec;

use iced::Element;
use iced::futures::TryFutureExt;
use iced::widget::{row, text_editor};
use sqlx::{FromRow, Pool, Sqlite};

use crate::app;
use crate::layout::{swim_lane, task_card, task_dialog, task_dialog_mut};

#[derive(Clone, Debug, Default, FromRow)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub lane: String,
}

#[derive(Clone, Debug)]
pub enum Message {
    MoveToLane(String, i64),
    RemoveTask(String, i64),
    TasksLoaded(Result<Vec<Task>, String>),
    CreateTask,
    EditTask(i64),
    TaskTitleUpdated(String),
    TaskDescUpdated(text_editor::Action),
    OpenModal(Modal),
    CloseModal,
}

#[derive(Clone, Debug)]
pub enum Modal {
    NewTask,
    ViewTask(i64),
    EditTask(i64),
}

pub struct ViewController {
    modal: Option<Modal>,
    db: Option<Pool<Sqlite>>,
    lanes: Vec<String>,
    tasks: Vec<Task>,
    next_id: i64,
    new_task_title: String,
    new_task_description: text_editor::Content,
}

impl Task {
    pub fn new(id: i64, title: String, description: Option<String>, lane: String) -> Self {
        Task {
            id,
            title,
            description,
            lane,
        }
    }
}

impl ViewController {
    pub fn new() -> Self {
        Self {
            modal: None,
            db: None,
            lanes: vec![],
            tasks: vec![],
            next_id: 1,
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

    fn remove_task(&mut self, lane: String, task_id: i64) {
        if let Some(pos) = self
            .tasks
            .iter()
            .position(|t| t.lane == lane && t.id == task_id)
        {
            self.tasks.remove(pos);
        }
    }

    pub fn configure(&mut self, conf: &app::Config) {
        let lanes = conf.lanes.clone();
        self.lanes = lanes;
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::MoveToLane(new_lane, task_id) => {
                if let Some(task) = self.find_task_by_id_mut(task_id) {
                    task.lane = new_lane;
                }
            }
            Message::RemoveTask(lane, task_id) => self.remove_task(lane, task_id),
            Message::TasksLoaded(tasks) => match tasks {
                Ok(tasks) => {
                    println!("loaded {} tasks", tasks.len());
                    self.tasks = tasks
                }
                Err(err) => println!("{err}"),
            },
            Message::CreateTask => {
                let title = self.new_task_title.clone();
                let desc = Some(self.new_task_description.text());
                if let Some(lane) = self.lanes.get(0) {
                    let task = Task::new(self.next_id, title, desc, lane.clone());
                    self.tasks.push(task);
                }
                self.next_id += 1;
                self.hide_dialog();
            }
            Message::EditTask(task_id) => {
                let title = self.new_task_title.clone();
                let desc = self.new_task_description.text();
                if let Some(task) = self.find_task_by_id_mut(task_id) {
                    task.title = title;
                    task.description = Some(desc);
                }
                self.hide_dialog();
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
            }
            Message::CloseModal => self.hide_dialog(),
            Message::TaskTitleUpdated(task_text) => self.new_task_title = task_text,
            Message::TaskDescUpdated(action) => self.new_task_description.perform(action),
        }
    }

    pub fn view(&self) -> Element<Message> {
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
                        Message::RemoveTask(t.lane.clone(), t.id),
                        Message::OpenModal(Modal::ViewTask(t.id)),
                        next_lane,
                    )
                })
                .collect();

            let title = format!("{} ({})", lane, tasks.len());
            swim_lane(title, elems)
        });

        row(lanes).spacing(24).into()
    }

    pub fn modal_view(&self) -> Option<Element<Message>> {
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

pub async fn get_tasks(pool: Pool<Sqlite>) -> Result<Vec<Task>, String> {
    sqlx::query_as!(Task, "SELECT id, title, description, lane FROM tasks")
        .fetch_all(&pool)
        .map_err(|err| format!("got db err: {err}"))
        .await
}
