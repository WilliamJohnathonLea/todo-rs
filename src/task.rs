use std::collections::HashMap;
use std::vec;

use iced::Element;
use iced::widget::{row, text_editor};

use crate::app::Config;
use crate::layout::{new_task_dialog, swim_lane, task_card, view_task_dialog};

#[derive(Default)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub lane: String,
}

#[derive(Clone, Debug)]
pub enum Message {
    MoveToLane(String, u32),
    RemoveTask(String, u32),
    CreateTask,
    TaskTitleUpdated(String),
    TaskDescUpdated(text_editor::Action),
    OpenModal(Modal),
    CloseModal,
}

#[derive(Clone, Debug)]
pub enum Modal {
    NewTask,
    ViewTask(u32),
}

pub struct ViewController {
    modal: Option<Modal>,
    lanes: Vec<String>,
    tasks: Vec<Task>,
    next_id: u32,
    new_task_title: String,
    new_task_description: text_editor::Content,
}

impl Task {
    pub fn new(id: u32, title: String, description: String) -> Self {
        Task {
            id,
            title,
            description,
            lane: crate::app::TO_DO.into(),
        }
    }
}

impl ViewController {
    pub fn new(config: &Config) -> Self {
        Self {
            modal: None,
            lanes: config.lanes.clone(),
            tasks: vec![Task::new(999, "Test".into(), "This is a test".into())],
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

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::MoveToLane(new_lane, task_id) => {
                if let Some(task) = self.find_task_by_id_mut(task_id) {
                    task.lane = new_lane;
                }
            }
            Message::RemoveTask(lane, task_id) => self.remove_task(lane, task_id),
            Message::CreateTask => {
                let title = self.new_task_title.clone();
                let desc = self.new_task_description.text();
                let task = Task::new(self.next_id, title, desc);
                self.tasks.push(task);
                self.next_id += 1;
                self.hide_dialog();
            }
            Message::OpenModal(modal) => self.modal = Some(modal),
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
                maybe_task.map(|t| view_task_dialog(t))
            }
            Some(Modal::NewTask) => Some(new_task_dialog(
                &self.new_task_title,
                &self.new_task_description,
                &Message::TaskTitleUpdated,
                &Message::TaskDescUpdated,
                Message::CreateTask,
                Message::CloseModal,
            )),
            None => None,
        }
    }
}
