CREATE TABLE problem (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255),
    description TEXT
);

CREATE TABLE solution (
    id SERIAL PRIMARY KEY,
    problem_id INT REFERENCES problem(id),
    code TEXT,
    language VARCHAR(50)
);

-- Seed Data
INSERT INTO problem (title, description) VALUES
('Hello World!', 'Write a program that prints "Hello, World!" to the console.');

INSERT INTO solution (problem_id, code, language) VALUES
(1, '', 'Python'),
(1, '', 'JavaScript'),
(1, '', 'C++');
