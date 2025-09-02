use std::collections::HashMap;
use std::vec;

use iced::Element;
use iced::widget::{button, row, text_editor};

use crate::app::{Config, ModalType};
use crate::layout::{swim_lane, task_card};

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
    OpenModal(ModalType),
}

pub struct ViewController {
    lanes: Vec<String>,
    tasks: Vec<Task>,
    next_id: u32,
    new_task_text: String,
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

    // fn move_button(&self) -> Element<Message> {
    //     if self.lane == crate::app::TO_DO {
    //         button(">")
    //             .on_press(Message::MoveToLane(crate::app::IN_PROGRESS.into(), self.id))
    //             .into()
    //     } else {
    //         button(">")
    //             .on_press(Message::MoveToLane(crate::app::DONE.into(), self.id))
    //             .into()
    //     }
    // }
}

impl ViewController {
    pub fn new(config: &Config) -> Self {
        Self {
            lanes: config.lanes.clone(),
            tasks: vec![],
            next_id: 1,
            new_task_text: Default::default(),
            new_task_description: Default::default(),
        }
    }

    fn hide_dialog(&mut self) {
        self.new_task_text.clear();
        self.new_task_description = text_editor::Content::new();
        //self.modal_type = ModalType::None;
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
                let title = self.new_task_text.clone();
                let desc = self.new_task_description.text();
                let task = Task::new(self.next_id, title, desc);
                self.tasks.push(task);
                self.next_id += 1;
                self.hide_dialog();
            }
            Message::OpenModal(_) => todo!(),
            Message::TaskTitleUpdated(task_text) => self.new_task_text = task_text,
            Message::TaskDescUpdated(action) => self.new_task_description.perform(action),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut grouped_by_lane: HashMap<&str, Vec<&Task>> = HashMap::new();

        for task in &self.tasks {
            grouped_by_lane.entry(&task.lane).or_default().push(task);
        }

        let lanes = self.lanes.iter().map(|lane| {
            let tasks = grouped_by_lane.remove(lane.as_str()).unwrap_or_default();
            let elems = tasks
                .iter()
                .map(|t| {
                    task_card(
                        t,
                        Message::RemoveTask(t.lane.clone(), t.id),
                        Message::OpenModal(ModalType::None),
                    )
                })
                .collect();
            let title = format!("{} ({})", lane, tasks.len());
            swim_lane(title, elems)
        });

        row(lanes).spacing(24).into()
    }
}
