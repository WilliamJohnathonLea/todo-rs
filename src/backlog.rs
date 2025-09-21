use crate::layout;
use crate::task::Task;
use crate::view_controller::ViewController as VC;

use iced::Length;
use iced::futures::TryFutureExt;
use iced::widget::{button, column, text};
use sqlx::{Pool, Sqlite};

#[derive(Clone, Debug)]
pub enum Message {
    OpenSprint,
    TasksLoaded(Result<Vec<Task>, String>),
}

pub struct ViewController {
    tasks: Vec<Task>,
}

impl ViewController {
    pub fn new() -> Self {
        ViewController { tasks: vec![] }
    }
}

impl VC for ViewController {
    type Message = Message;

    fn update(&mut self, msg: Self::Message) -> iced::Task<Self::Message> {
        match msg {
            Message::OpenSprint => iced::Task::none(), // Handled at the App level
            Message::TasksLoaded(result) => {
                if let Ok(tasks) = result {
                    self.tasks = tasks;
                }
                iced::Task::none()
            }
        }
    }

    fn view(&self) -> iced::Element<Self::Message> {
        let mut task_views = vec![];
        for task in self.tasks.iter() {
            task_views.push(text(&task.title).into());
        }

        column![
            button("Sprint").on_press(Message::OpenSprint),
            text("Backlog").size(24),
            layout::backlog(task_views)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(4)
        .into()
    }
}

pub async fn get_tasks(pool: Pool<Sqlite>) -> Result<Vec<Task>, String> {
    sqlx::query_as!(
        Task,
        "SELECT id, title, description, lane FROM tasks WHERE in_backlog"
    )
    .fetch_all(&pool)
    .map_err(|err| format!("got db err: {err}"))
    .await
}
