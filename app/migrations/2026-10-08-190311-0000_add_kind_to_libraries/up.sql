ALTER TABLE libraries ADD COLUMN kind TEXT NOT NULL DEFAULT 'comic' CHECK (kind IN ('manga', 'comic', 'ebook'));
