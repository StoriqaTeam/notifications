CREATE TABLE templates (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    data VARCHAR NOT NULL    
);

ALTER TABLE templates
ADD CONSTRAINT unique_name UNIQUE (name);