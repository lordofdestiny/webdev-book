CREATE TABLE IF NOT EXISTS answers
(
    id          SERIAL PRIMARY KEY,
    content     TEXT      NOT NULL,
    created_on  TIMESTAMP NOT NULL DEFAULT NOW(),
    question_id INTEGER REFERENCES questions
);
