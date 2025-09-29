use crate::layout;
use crate::task::Task;
use crate::view_controller::ViewController as VC;

use iced::Length;
use iced::futures::TryFutureExt;
use iced::widget::{button, column, container, horizontal_space, row, text};
use sqlx::{Pool, Sqlite};

#[derive(Clone, Debug)]
pub enum Message {
    OpenSprint,
    TasksLoaded(Result<Vec<Task>, String>),
    RemoveTask(i64),
    NoOp,
}

pub struct ViewController {
    db: Pool<Sqlite>,
    tasks: Vec<Task>,
}

impl ViewController {
    pub fn new(db: Pool<Sqlite>) -> (Self, iced::Task<Message>) {
        (
            ViewController {
                db: db.clone(),
                tasks: vec![],
            },
            iced::Task::perform(get_tasks(db), Message::TasksLoaded),
        )
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
            Message::RemoveTask(task_id) => {
                iced::Task::perform(remove_task(self.db.clone(), task_id), |_| Message::NoOp).chain(
                    iced::Task::perform(get_tasks(self.db.clone()), Message::TasksLoaded),
                )
            }
            Message::NoOp => iced::Task::none(),
        }
    }

    fn view(&self) -> iced::Element<Self::Message> {
        let mut task_views = vec![];
        for task in self.tasks.iter() {
            let item = container(row![
                text(format!("{}: {}", task.id, task.title)),
                horizontal_space(),
                button("X").on_press(Message::RemoveTask(task.id))
            ])
            .style(container::bordered_box);
            task_views.push(item.into());
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

async fn get_tasks(pool: Pool<Sqlite>) -> Result<Vec<Task>, String> {
    sqlx::query_as!(
        Task,
        "SELECT id, title, description, lane FROM tasks WHERE in_backlog"
    )
    .fetch_all(&pool)
    .map_err(|err| format!("got db err: {err}"))
    .await
}

async fn remove_task(pool: Pool<Sqlite>, task_id: i64) -> Result<(), String> {
    sqlx::query!("DELETE FROM tasks WHERE id = ?", task_id)
        .execute(&pool)
        .map_err(|_| "Error deleting task from db".into())
        .map_ok(|_| ())
        .await
}
