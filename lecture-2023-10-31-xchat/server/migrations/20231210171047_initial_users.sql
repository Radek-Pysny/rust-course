DELETE FROM users
WHERE login IN ('TheOne', 'JustTwo', 'Threesome');

INSERT INTO users
(login, password)
VALUES
-- password = MD5 of string '1'
('TheOne', 'c4ca4238a0b923820dcc509a6f75849b'),
-- password = MD5 of string '2'
('JustTwo', 'c81e728d9d4c2f636f067f89cc14862c'),
-- password = MD5 of string '3'
('Threesome', 'eccbc87e4b5ce2fe28308fd9f2a7baf3');
