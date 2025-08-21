use crate::app::Message;
use crate::app::ModalType::ViewTask;
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

pub fn swim_lane<'a>(title: &'a str, tasks: &'a [Task]) -> Element<'a, Message> {
    let lane_title = text!("{} ({})", title, tasks.len()).size(24);
    let mut content = column!(lane_title).spacing(8).width(Length::Fill);

    for t in tasks {
        content = content.push(task_card(t));
    }

    content.into()
}

pub fn new_task_dialog<'a>(
    task_title: &'a str,
    task_description: &'a text_editor::Content,
) -> Element<'a, Message> {
    column![
        text("New Task").size(24),
        text("Title"),
        text_input("", task_title)
            .on_input(Message::TaskTitleUpdated)
            .on_paste(Message::TaskTitleUpdated)
            .on_submit(Message::SubmitTask),
        text("Description"),
        text_editor(task_description).on_action(Message::TaskDescUpdated),
        row![
            button("Add Task").on_press(Message::SubmitTask),
            button("Cancel").on_press(Message::CloseDialog)
        ]
        .spacing(8)
    ]
    .spacing(8)
    .align_x(Horizontal::Center)
    .into()
}

pub fn view_task_dialog<'a>(task: &'a Task) -> Element<'a, Message> {
    column![text(&task.title), text(&task.description)].into()
}

fn task_card<'a>(t: &'a Task) -> Element<'a, Message> {
    let card_content = row![
        column![text(&t.title).size(20), text(t.id)].width(Length::Fill),
        button("X").on_press(Message::RemoveTask)
    ];

    let card = container(card_content)
        .style(container::rounded_box)
        .padding(8)
        .width(Length::Fill);

    mouse_area(card)
        .on_press(Message::OpenDialog(ViewTask(t.id)))
        .into()
}
