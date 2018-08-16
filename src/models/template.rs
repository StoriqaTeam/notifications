//! Models for managing Templates

use repos::TemplateVariant;
use schema::templates;

#[derive(Serialize, Queryable, Insertable, Debug)]
#[table_name = "templates"]
pub struct Template {
    pub id: i32,
    pub name: TemplateVariant,
    pub data: String,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "templates"]
pub struct NewTemplate {
    pub name: TemplateVariant,
    pub data: String,
}
