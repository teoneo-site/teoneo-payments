use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub user_id: i64,
    pub course_id: i64,
}
