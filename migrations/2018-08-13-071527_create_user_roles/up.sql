-- Your SQL goes here
CREATE TABLE user_roles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    role VARCHAR NOT NULL,
);

CREATE UNIQUE INDEX user_roles_user_id_idx ON user_roles (user_id);

SELECT diesel_manage_updated_at('user_roles');
