use std::{
    fs::{self, File},
    path::Path,
};

use rusqlite::{Connection, Result};

#[derive(Debug)]
pub struct BookRecord {
    id: Option<i32>,
    pub url: String,
}

#[derive(Debug, Default)]
pub struct Storage {
    db_path: Option<String>,
}

impl Storage {
    pub fn new(path: Option<String>) -> Self {
        Self { db_path: path }
    }
    pub fn db_path(&self) -> String {
        if let Some(path) = &self.db_path {
            return path.clone();
        }
        Self::default_db_path().into()
    }

    pub fn default_db_path() -> String {
        let home = dirs::home_dir().unwrap();
        let db_path = home.join(".oreally/oreilly.db");
        db_path.to_str().unwrap().to_string()
    }

    pub fn conn(&self) -> Result<Connection, Box<dyn std::error::Error>> {
        if !self.is_ready() {
            return Err("Database is not ready".into());
        }
        let db_path = self.db_path();
        let conn = Connection::open(db_path)?;
        Ok(conn)
    }
    pub fn is_ready(&self) -> bool {
        let db_path = self.db_path();
        Path::new(&db_path).exists()
    }
    pub fn setup(&self) -> Result<(), Box<dyn std::error::Error>> {
        let db_path = self.db_path();
        if Path::new(&db_path).exists() {
            println!("Database file already exists at {db_path:?}");
            return Ok(());
        }
        let folder = Path::new(&db_path).parent().unwrap();
        fs::create_dir_all(folder).unwrap_or_else(|_| {
            panic!("unable to create folder {folder:?} for database file");
        });
        File::create(&db_path)
            .unwrap_or_else(|_| panic!("unable to create database file at {db_path:?}"));
        let conn = self.conn()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS book_queue (
                id    INTEGER PRIMARY KEY,
                url  TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }
}

impl BookRecord {
    pub fn new(url: String) -> Self {
        Self { id: None, url }
    }
    pub fn insert(&mut self, storage: &Storage) -> Result<&mut Self, Box<dyn std::error::Error>> {
        if self.id.is_some() {
            return Ok(self);
        }
        let conn = storage.conn()?;
        println!("Inserting book {:?}", self);
        conn.execute("INSERT INTO book_queue (url) VALUES (?1)", [&self.url])?;

        self.id = Some(conn.last_insert_rowid().try_into()?);
        println!("Inserted book with id {:#?}", self);
        Ok(self)
    }
    pub fn all(storage: &Storage) -> Result<Vec<BookRecord>, Box<dyn std::error::Error>> {
        let conn = storage.conn()?;
        let mut stmt = conn.prepare("SELECT id, url FROM book_queue")?;
        let book_iter = stmt.query_map([], |row| {
            Ok(BookRecord {
                id: row.get(0)?,
                url: row.get(1)?,
            })
        })?;

        let mut books = Vec::new();
        for book in book_iter {
            books.push(book.unwrap());
        }
        Ok(books)
    }

    pub fn delete(&self, storage: &Storage) -> Result<(), Box<dyn std::error::Error>> {
        if self.id.is_none() {
            return Ok(());
        }
        let conn = storage.conn()?;
        conn.execute("DELETE FROM book_queue WHERE id = ?1", [self.id])?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn book_record_new() {
        todo!()
    }
    #[test]
    fn book_record_insert() {
        todo!()
    }
    #[test]
    fn book_record_all() {
        todo!()
    }
    #[test]
    fn book_record_delete() {
        todo!()
    }
    #[test]
    fn storage_db_path() {
        todo!()
    }
    #[test]
    fn storage_conn() {
        todo!()
    }
    #[test]
    fn storage_is_ready() {
        todo!()
    }
    #[test]
    fn storage_setup() {
        todo!()
    }
}
