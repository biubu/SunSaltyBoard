use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: String,
    pub content_type: String,
    pub content: String,
    pub preview: String,
    pub group_id: Option<String>,
    pub created_at: String,
    pub is_favorite: bool,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub color: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkey {
    pub id: String,
    pub action: String,
    pub key_combination: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub config: Option<String>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(app: &AppHandle) -> Result<Self> {
        let app_dir = app
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        std::fs::create_dir_all(&app_dir)?;

        let db_path = app_dir.join("clipstash.db");
        log::info!("Database path: {:?}", db_path);

        let conn = Connection::open(&db_path)?;
        let db = Database { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS clipboard_items (
                id TEXT PRIMARY KEY,
                content_type TEXT NOT NULL,
                content TEXT NOT NULL,
                preview TEXT NOT NULL,
                group_id TEXT,
                created_at TEXT NOT NULL,
                is_favorite INTEGER DEFAULT 0,
                metadata TEXT
            );

            CREATE TABLE IF NOT EXISTS groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS item_tags (
                item_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                PRIMARY KEY (item_id, tag_id),
                FOREIGN KEY (item_id) REFERENCES clipboard_items(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS hotkeys (
                id TEXT PRIMARY KEY,
                action TEXT NOT NULL UNIQUE,
                key_combination TEXT NOT NULL,
                enabled INTEGER DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS plugins (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                enabled INTEGER DEFAULT 1,
                config TEXT
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS clipboard_fts USING fts5(
                content,
                content='clipboard_items',
                content_rowid='rowid'
            );

            CREATE TRIGGER IF NOT EXISTS clipboard_ai AFTER INSERT ON clipboard_items BEGIN
                INSERT INTO clipboard_fts(rowid, content) VALUES (new.rowid, new.content);
            END;

            CREATE TRIGGER IF NOT EXISTS clipboard_ad AFTER DELETE ON clipboard_items BEGIN
                INSERT INTO clipboard_fts(clipboard_fts, rowid, content) VALUES('delete', old.rowid, old.content);
            END;

            CREATE TRIGGER IF NOT EXISTS clipboard_au AFTER UPDATE ON clipboard_items BEGIN
                INSERT INTO clipboard_fts(clipboard_fts, rowid, content) VALUES('delete', old.rowid, old.content);
                INSERT INTO clipboard_fts(rowid, content) VALUES (new.rowid, new.content);
            END;

            CREATE INDEX IF NOT EXISTS idx_items_created_at ON clipboard_items(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_items_group_id ON clipboard_items(group_id);
            CREATE INDEX IF NOT EXISTS idx_items_favorite ON clipboard_items(is_favorite);
            CREATE INDEX IF NOT EXISTS idx_items_content_type ON clipboard_items(content_type);
            "#,
        )?;
        Ok(())
    }

    pub fn insert_clipboard_item(&self, item: &ClipboardItem) -> Result<()> {
        self.conn.execute(
            "INSERT INTO clipboard_items (id, content_type, content, preview, group_id, created_at, is_favorite, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                item.id,
                item.content_type,
                item.content,
                item.preview,
                item.group_id,
                item.created_at,
                item.is_favorite as i32,
                item.metadata,
            ],
        )?;
        Ok(())
    }

    pub fn get_clipboard_history(&self, limit: i32, offset: i32) -> Result<Vec<ClipboardItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_type, content, preview, group_id, created_at, is_favorite, metadata
             FROM clipboard_items
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let items = stmt
            .query_map(params![limit, offset], |row| {
                Ok(ClipboardItem {
                    id: row.get(0)?,
                    content_type: row.get(1)?,
                    content: row.get(2)?,
                    preview: row.get(3)?,
                    group_id: row.get(4)?,
                    created_at: row.get(5)?,
                    is_favorite: row.get::<_, i32>(6)? != 0,
                    metadata: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(items)
    }

    pub fn search_clipboard(&self, query: &str, limit: i32) -> Result<Vec<ClipboardItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.content_type, c.content, c.preview, c.group_id, c.created_at, c.is_favorite, c.metadata
             FROM clipboard_items c
             JOIN clipboard_fts f ON c.rowid = f.rowid
             WHERE clipboard_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;

        let items = stmt
            .query_map(params![query, limit], |row| {
                Ok(ClipboardItem {
                    id: row.get(0)?,
                    content_type: row.get(1)?,
                    content: row.get(2)?,
                    preview: row.get(3)?,
                    group_id: row.get(4)?,
                    created_at: row.get(5)?,
                    is_favorite: row.get::<_, i32>(6)? != 0,
                    metadata: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(items)
    }

    pub fn delete_item(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM clipboard_items WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_item_group(&self, item_id: &str, group_id: Option<&str>) -> Result<()> {
        self.conn.execute(
            "UPDATE clipboard_items SET group_id = ?1 WHERE id = ?2",
            params![group_id, item_id],
        )?;
        Ok(())
    }

    pub fn toggle_favorite(&self, id: &str) -> Result<bool> {
        self.conn.execute(
            "UPDATE clipboard_items SET is_favorite = NOT is_favorite WHERE id = ?1",
            params![id],
        )?;
        let is_favorite: i32 = self.conn.query_row(
            "SELECT is_favorite FROM clipboard_items WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        Ok(is_favorite != 0)
    }

    // Groups
    pub fn get_groups(&self) -> Result<Vec<Group>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, color, created_at FROM groups ORDER BY name")?;

        let groups = stmt
            .query_map([], |row| {
                Ok(Group {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(groups)
    }

    pub fn create_group(&self, name: &str, color: &str) -> Result<Group> {
        let id = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO groups (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, color, created_at],
        )?;

        Ok(Group {
            id,
            name: name.to_string(),
            color: color.to_string(),
            created_at,
        })
    }

    pub fn delete_group(&self, id: &str) -> Result<()> {
        // Move items in this group to no group
        self.conn.execute(
            "UPDATE clipboard_items SET group_id = NULL WHERE group_id = ?1",
            params![id],
        )?;
        self.conn
            .execute("DELETE FROM groups WHERE id = ?1", params![id])?;
        Ok(())
    }

    // Tags
    pub fn get_tags(&self) -> Result<Vec<Tag>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, color FROM tags ORDER BY name")?;

        let tags = stmt
            .query_map([], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tags)
    }

    pub fn create_tag(&self, name: &str, color: &str) -> Result<Tag> {
        let id = Uuid::new_v4().to_string();

        self.conn.execute(
            "INSERT INTO tags (id, name, color) VALUES (?1, ?2, ?3)",
            params![id, name, color],
        )?;

        Ok(Tag {
            id,
            name: name.to_string(),
            color: color.to_string(),
        })
    }

    pub fn delete_tag(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM item_tags WHERE tag_id = ?1", params![id])?;
        self.conn
            .execute("DELETE FROM tags WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn add_tag_to_item(&self, item_id: &str, tag_id: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO item_tags (item_id, tag_id) VALUES (?1, ?2)",
            params![item_id, tag_id],
        )?;
        Ok(())
    }

    pub fn remove_tag_from_item(&self, item_id: &str, tag_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM item_tags WHERE item_id = ?1 AND tag_id = ?2",
            params![item_id, tag_id],
        )?;
        Ok(())
    }

    pub fn get_item_tags(&self, item_id: &str) -> Result<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name, t.color FROM tags t
             JOIN item_tags it ON t.id = it.tag_id
             WHERE it.item_id = ?1",
        )?;

        let tags = stmt
            .query_map(params![item_id], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tags)
    }

    // Hotkeys
    pub fn get_hotkeys(&self) -> Result<Vec<Hotkey>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, action, key_combination, enabled FROM hotkeys")?;

        let hotkeys = stmt
            .query_map([], |row| {
                Ok(Hotkey {
                    id: row.get(0)?,
                    action: row.get(1)?,
                    key_combination: row.get(2)?,
                    enabled: row.get::<_, i32>(3)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(hotkeys)
    }

    pub fn update_hotkey(&self, action: &str, key_combination: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO hotkeys (id, action, key_combination, enabled)
             VALUES ((SELECT id FROM hotkeys WHERE action = ?1), ?1, ?2, 1)",
            params![action, key_combination],
        )?;
        Ok(())
    }

    // Settings
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }
}
