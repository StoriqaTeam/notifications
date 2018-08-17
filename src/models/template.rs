//! Models for managing Templates

use schema::templates;
use stq_static_resources::TemplateVariant;

#[derive(Serialize, Deserialize, PartialEq, Eq, Queryable, Insertable, Debug)]
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
