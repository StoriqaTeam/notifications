table! {
    templates (id) {
        id -> Int4,
        name -> Varchar,
        data -> Varchar,
    }
}

table! {
    user_roles (id) {
        user_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        name -> Varchar,
        data -> Nullable<Jsonb>,
        id -> Uuid,
    }
}

allow_tables_to_appear_in_same_query!(
    templates,
    user_roles,
);
