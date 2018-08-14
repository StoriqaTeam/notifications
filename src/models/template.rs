//! Models for managing Templates

use schema::templates;

#[derive(Serialize, Queryable, Insertable, Debug)]
#[table_name = "templates"]
pub struct Template {
    pub id: i32,
    pub name: String,
    pub data: String,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "templates"]
pub struct NewTemplate {
    pub name: String,
    pub data: String,
}

/// Payload for updating templaes
#[derive(Serialize, Deserialize, Insertable, AsChangeset, Clone, Debug)]
#[table_name = "templates"]
pub struct UpdateTemplate {
    pub name: String,
    pub data: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OldTemplate {
    pub name: String,
}
