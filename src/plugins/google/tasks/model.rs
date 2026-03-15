use serde::Deserialize;

#[derive(Deserialize)]
pub struct TaskListsResponse {
    pub items: Option<Vec<TaskListItem>>,
}

#[derive(Deserialize)]
pub struct TaskListItem {
    pub id: String,
    pub title: String,
}

#[derive(Deserialize)]
pub struct TasksResponse {
    pub items: Option<Vec<TaskItem>>,
}

#[derive(Deserialize)]
pub struct TaskItem {
    pub id: String,
    pub title: Option<String>,
    pub notes: Option<String>,
    // pub status: Option<String>,
    pub due: Option<String>,
    // pub completed: Option<String>,
    pub updated: Option<String>,
    pub parent: Option<String>,
    #[serde(rename = "selfLink")]
    pub self_link: Option<String>,
}
