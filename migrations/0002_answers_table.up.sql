CREATE TABLE IF NOT EXISTS answers
(
    id          SERIAL PRIMARY KEY,
    content     TEXT      NOT NULL,
    created_on  TIMESTAMP NOT NULL DEFAULT NOW(),
    question_id INTEGER REFERENCES questions
);

INSERT INTO answers (content, question_id)
VALUES ('It''s really easy! Just run CREATE TABLE table_name ( ... )', 1)
ON CONFLICT DO NOTHING;

INSERT INTO answers (content, question_id)
VALUES ('Maybe Google the documentation?', 1)
ON CONFLICT DO NOTHING;

INSERT INTO answers (content, question_id)
VALUES ('Google before asking!', 2)
ON CONFLICT DO NOTHING;

INSERT INTO answers (content, question_id)
VALUES ('Hello world! Go kill yourself!!!', 2)
ON CONFLICT DO NOTHING;
