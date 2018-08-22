table! {
    templates (id) {
        id -> Int4,
        name -> Varchar,
        data -> Varchar,
    }
}

table! {
    user_roles (id) {
        id -> Int4,
        user_id -> Int4,
        role -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(templates, user_roles,);
