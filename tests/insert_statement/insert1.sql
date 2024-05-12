CREATE TABLE test;
ALTER TABLE test ADD text VARCHAR(255);
INSERT INTO test (text) VALUES ('hello1');
SHOW TABLES;
ALTER TABLE test ADD value INT;
INSERT INTO test (text,value) VALUES ('hello2', 1);
SHOW TABLES;