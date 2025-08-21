#[derive(Default)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub description: String,
}

impl Task {
    pub fn new(id: u32, title: String, description: String) -> Self {
        Task {
            id,
            title,
            description,
        }
    }
}
