use crate::app::Message;
use crate::task::Task;
use iced::alignment::Horizontal;
use iced::widget::{
    button, center, column, container, mouse_area, opaque, row, stack, text, text_editor,
    text_input,
};
use iced::{Color, Element, Length};

pub fn modal<'a>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message> {
    stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|_theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.8,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            }))
            .on_press(on_blur)
        )
    ]
    .into()
}

pub fn swim_lane<'a>(title: String, tasks: Vec<&'a Task>) -> Element<'a, Message> {
    let lane_title = text!("{} ({})", title, tasks.len()).size(24);
    let mut content = column!(lane_title).spacing(8).width(Length::Fill);

    for t in tasks {
        content = content.push(t.view().map(|msg| Message::TaskMessage(msg)));
    }

    content.into()
}

pub fn new_task_dialog<'a>(
    task_title: &'a str,
    task_description: &'a text_editor::Content,
) -> Element<'a, Message> {
    let content = column![
        text("New Task").size(24),
        text("Title"),
        text_input("", task_title)
            .on_input(Message::TaskTitleUpdated)
            .on_paste(Message::TaskTitleUpdated)
            .on_submit(Message::SubmitTask),
        text("Description"),
        text_editor(task_description)
            .height(Length::Fill)
            .on_action(Message::TaskDescUpdated),
        row![
            button("Add Task").on_press(Message::SubmitTask),
            button("Cancel").on_press(Message::CloseDialog)
        ]
        .spacing(8)
    ]
    .spacing(8)
    .align_x(Horizontal::Center);

    container(content)
        .style(container::bordered_box)
        .padding([16, 16])
        .into()
}

pub fn view_task_dialog<'a>(task: &'a Task) -> Element<'a, Message> {
    let content = column![text(&task.title), text(&task.description)];
    container(content)
        .style(container::bordered_box)
        .padding([16, 16])
        .into()
}
