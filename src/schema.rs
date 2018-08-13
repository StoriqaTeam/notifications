table! {
    templates (id) {
        id -> Integer,
        name -> VarChar,
        data -> VarChar,
    }
}

table! {
    user_roles (id) {
        id -> Integer,
        user_id -> Integer,
        role -> VarChar,
    }
}
