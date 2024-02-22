CREATE TABLE IF NOT EXISTS answers
(
    id          SERIAL PRIMARY KEY,
    content     TEXT      NOT NULL,
    created_on  TIMESTAMP NOT NULL DEFAULT NOW(),
    question_id INTEGER REFERENCES questions
);

INSERT INTO answers (id, content, question_id)
VALUES (1, 'It''s really easy! Just run CREATE TABLE table_name ( ... )', 1)
ON CONFLICT DO NOTHING;

INSERT INTO answers (id, content, question_id)
VALUES (2, 'Maybe Google the documentation?', 1)
ON CONFLICT DO NOTHING;

INSERT INTO answers (id, content, question_id)
VALUES (3, 'Google before asking!', 2)
ON CONFLICT DO NOTHING;

INSERT INTO answers (id, content, question_id)
VALUES (4, 'Hello world! Go kill yourself!!!', 2)
ON CONFLICT DO NOTHING;
