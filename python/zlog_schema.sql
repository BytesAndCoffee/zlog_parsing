CREATE TABLE `event_log` (
  `id` INT NOT NULL,
  `user` VARCHAR(128) DEFAULT NULL,
  `network` VARCHAR(128) NOT NULL,
  `window` VARCHAR(255) NOT NULL,
  `type` VARCHAR(32) NOT NULL,
  `nick` VARCHAR(128) DEFAULT NULL,
  `message` TEXT,
  `recipient` VARCHAR(64) DEFAULT 'self',
  PRIMARY KEY (`id`),
  KEY (`user`)
);

CREATE TABLE `inbound` (
  `id` INT NOT NULL AUTO_INCREMENT,
  `user` VARCHAR(128) NOT NULL,
  `network` VARCHAR(128) NOT NULL,
  `window` VARCHAR(255) NOT NULL,
  `type` VARCHAR(32) NOT NULL,
  `nick` VARCHAR(128) NOT NULL,
  `message` TEXT NOT NULL,
  `tg_id` INT NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `inbound_tg_id_uindex` (`tg_id`)
);

CREATE TABLE `inbound_log` (
  `id` INT NOT NULL,
  `user` VARCHAR(128) NOT NULL,
  `network` VARCHAR(128) NOT NULL,
  `window` VARCHAR(255) NOT NULL,
  `type` VARCHAR(32) NOT NULL,
  `nick` VARCHAR(128) NOT NULL,
  `message` TEXT NOT NULL,
  `tg_id` INT NOT NULL,
  PRIMARY KEY (`id`)
);

CREATE TABLE `lastread` (
  `table` VARCHAR(32) NOT NULL,
  `id` INT DEFAULT NULL,
  PRIMARY KEY (`table`),
  UNIQUE KEY `lastread_table_uindex` (`table`)
);

CREATE TABLE `logs` (
  `id` INT NOT NULL AUTO_INCREMENT,
  `created_at` DATETIME NOT NULL,
  `user` VARCHAR(128) DEFAULT NULL,
  `network` VARCHAR(128) DEFAULT NULL,
  `window` VARCHAR(255) NOT NULL,
  `type` VARCHAR(32) NOT NULL,
  `nick` VARCHAR(128) DEFAULT NULL,
  `message` TEXT,
  PRIMARY KEY (`id`),
  KEY `created_at` (`created_at`),
  KEY `user` (`user`),
  KEY `network` (`network`),
  KEY `nick_idx` (`nick`),
  KEY `window_idx` (`window`),
  KEY `window_nick_idx` (`window`,`nick`),
  KEY `type_idx` (`type`),
  KEY `analytics_idx` (`window`,`created_at`,`nick`)
);

CREATE TABLE `logs_queue` (
  `id` INT NOT NULL,
  `created_at` DATETIME NOT NULL,
  `user` VARCHAR(128) DEFAULT NULL,
  `network` VARCHAR(128) DEFAULT NULL,
  `window` VARCHAR(255) NOT NULL,
  `type` VARCHAR(32) NOT NULL,
  `nick` VARCHAR(128) DEFAULT NULL,
  `message` TEXT,
  PRIMARY KEY (`id`),
  KEY `created_at` (`created_at`),
  KEY `user` (`user`),
  KEY `network` (`network`),
  KEY `nick_idx` (`nick`),
  KEY `window_idx` (`window`),
  KEY `window_nick_idx` (`window`,`nick`),
  KEY `type_idx` (`type`)
);

CREATE TABLE `push` (
  `id` INT NOT NULL,
  `user` VARCHAR(128) DEFAULT NULL,
  `network` VARCHAR(128) NOT NULL,
  `window` VARCHAR(255) NOT NULL,
  `type` VARCHAR(32) NOT NULL,
  `nick` VARCHAR(128) DEFAULT NULL,
  `message` TEXT,
  `recipient` VARCHAR(64) DEFAULT 'self',
  PRIMARY KEY (`id`),
  KEY `user` (`user`)
);

CREATE TABLE `users` (
  `nickname` VARCHAR(64) NOT NULL,
  `telegram_chat_id` BIGINT DEFAULT NULL,
  `hotwords` JSON DEFAULT NULL,
  PRIMARY KEY (`nickname`)
);
