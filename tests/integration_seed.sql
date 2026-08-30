INSERT INTO logs_id_track (id, tid) VALUES (1, 1);
INSERT INTO users (nickname, telegram_chat_id, hotwords)
VALUES ('tester', 1, '[{"type":"substring","match":"alert"}]');
INSERT INTO logs (id, created_at, user, network, `window`, type, nick, message)
VALUES
  (1, '2026-08-30 10:00:00', 'tester', 'testnet', '#test', 'msg', 'alice', 'alert one'),
  (2, '2026-08-30 10:01:00', 'tester', 'testnet', '#test', 'msg', 'bob', 'ordinary'),
  (3, '2026-08-30 10:02:00', 'tester', 'testnet', '#test', 'msg', 'carol', 'alert two');
INSERT INTO logs_queue (id, created_at, user, network, `window`, type, nick, message)
VALUES (1, '2026-08-30 10:00:00', 'tester', 'testnet', '#test', 'msg', 'alice', 'alert one');
