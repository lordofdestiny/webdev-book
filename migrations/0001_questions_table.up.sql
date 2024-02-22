CREATE TABLE IF NOT EXISTS questions
(
    id         SERIAL PRIMARY KEY,
    title      VARCHAR(255) NOT NULL,
    content    TEXT         NOT NULL,
    tags        TEXT[],
    created_on TIMESTAMP    NOT NULL DEFAULT NOW()
);

INSERT INTO questions (id, title, content, tags)
VALUES (1, 'How to create a table in PostgreSQL?',
        'I am trying to create a table in PostgreSQL ' ||
        'but I am not able to do it.Can someone help me?',
        ARRAY ['PostgreSQL', 'SQL', 'Database'])
ON CONFLICT DO NOTHING;

INSERT INTO questions (id, title, content, tags)
VALUES (2, 'How to create a table in MySQL?',
        'I am trying to create a table in MySQL ' ||
        'but I am not able to do it.Can someone help me?',
        ARRAY ['MySQL', 'SQL', 'Database'])
ON CONFLICT DO NOTHING;