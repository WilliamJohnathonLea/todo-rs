use crate::task::Task;
use iced::alignment::Horizontal;
use iced::widget::{
    button, center, column, container, mouse_area, opaque, row, stack, text, text_editor,
    text_input,
};
use iced::{Color, Element, Length};

pub fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
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

pub fn swim_lane<'a, Message>(
    title: String,
    tasks: Vec<Element<'a, Message>>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let lane_title = text(title).size(24);
    let mut content = column!(lane_title).spacing(8).width(Length::Fill);

    for task in tasks {
        content = content.push(task);
    }

    content.into()
}

pub fn task_card<'a, Message>(
    task: &'a Task,
    remove: Message,
    open_modal: Message,
    next_lane: Option<Message>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let card_content = row![
        column![
            text(&task.title).size(20),
            text(task.id),
            button(">").on_press_maybe(next_lane),
        ]
        .width(Length::Fill),
        button("X").on_press(remove)
    ];

    let card = container(card_content)
        .style(container::rounded_box)
        .padding(8)
        .width(Length::Fill);

    mouse_area(card).on_press(open_modal).into()
}

pub fn new_task_dialog<'a, Message, TU, DU>(
    task_title: &'a str,
    task_description: &'a text_editor::Content,
    title_update: &'a TU,
    description_update: &'a DU,
    submit: Message,
    cancel: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    TU: Fn(String) -> Message + 'a,
    DU: Fn(text_editor::Action) -> Message + 'a,
{
    let content = column![
        text("New Task").size(24),
        text("Title"),
        text_input("", task_title)
            .on_input(title_update)
            .on_paste(title_update),
        text("Description"),
        text_editor(task_description)
            .height(Length::Fill)
            .on_action(description_update),
        row![
            button("Add Task").on_press(submit),
            button("Cancel").on_press(cancel)
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

pub fn view_task_dialog<'a, Message>(task: &'a Task) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let content = column![text(&task.title), text(&task.description)];
    container(content)
        .style(container::bordered_box)
        .padding([16, 16])
        .into()
}
