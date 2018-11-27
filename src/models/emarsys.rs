use stq_types::{EmarsysId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContactPayload {
    pub user_id: UserId,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedContact {
    pub user_id: UserId,
    pub emarsys_id: EmarsysId,
}
