use std::{
    fs::{self, File},
    path::Path,
};

use prettytable::row;
use rusqlite::{Connection, Result};

use crate::parse_url;

#[derive(Debug, PartialEq, Clone)]
pub struct BookRecord {
    id: Option<i32>,
    pub url: String,
}

#[derive(Debug, Default)]
pub struct Storage {
    db_path: Option<String>,
}

impl Storage {
    #[allow(dead_code)]
    pub fn drop(&self) -> Result<(), Box<dyn std::error::Error>> {
        let db_path = self.db_path();
        if Path::new(&db_path).exists() {
            fs::remove_file(&db_path)?;
        }
        Ok(())
    }
    #[allow(dead_code)]
    pub fn flush(&self) -> Result<(), Box<dyn std::error::Error>> {
        let db_path = self.db_path();
        if Path::new(&db_path).exists() {
            fs::remove_file(&db_path)?;
        }
        self.setup()
    }
    #[allow(dead_code)]
    pub fn new(path: Option<String>) -> Self {
        Self { db_path: path }
    }
    pub fn db_path(&self) -> String {
        if let Some(path) = &self.db_path {
            return path.clone();
        }
        Self::default_db_path()
    }

    pub fn default_db_path() -> String {
        let home = dirs::home_dir().unwrap();
        let db_path = home.join(".oreally/database.db");
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
            return Err("Database file already exists at {db_path:?}".into());
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
    pub fn new(url: &str) -> Self {
        Self {
            id: None,
            url: url.to_string(),
        }
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
    pub fn all(storage: &Storage) -> Vec<BookRecord> {
        let conn = storage.conn().unwrap();
        let mut stmt = conn.prepare("SELECT id, url FROM book_queue").unwrap();
        let book_iter = stmt
            .query_map([], |row| {
                Ok(BookRecord {
                    id: row.get(0)?,
                    url: row.get(1)?,
                })
            })
            .unwrap();

        let mut books = Vec::new();
        for book in book_iter {
            books.push(book.unwrap());
        }
        books
    }

    pub fn table(books: &Vec<Self>) -> Result<prettytable::Table> {
        let mut table = prettytable::Table::new();
        table.add_row(row!["ID", "Book ID", "Title", "URL"]);
        for book in books {
            let (tittle, oreilly_id) =
                parse_url(&book.url).unwrap_or_else(|_| ("".to_string(), "".to_string()));
            table.add_row(row![book.id.unwrap(), oreilly_id, tittle, book.url]);
        }
        Ok(table)
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
pub(crate) mod test {
    use super::*;
    use uuid::Uuid;

    pub fn around(f: impl Fn(&Storage)) {
        fs::create_dir_all("tmp").unwrap();
        let storage = Storage::new(Some(format!("tmp/{}.db", Uuid::new_v4()).to_string()));
        storage.flush().unwrap();
        f(&storage);
        storage.drop().unwrap();
    }

    #[test]
    fn book_record_new() {
        let url = "https://www.oreilly.com/library/view/".to_string();
        assert_eq!(BookRecord::new(&url), BookRecord { id: None, url });
    }
    #[test]
    fn book_record_insert() {
        around(|storage| {
            let url = "https://www.oreilly.com/library/view/".to_string();
            let mut r = BookRecord::new(&url);
            r.insert(&storage).unwrap();
            assert_eq!(r, BookRecord { id: Some(1), url });
        })
    }
    #[test]
    fn book_record_all() {
        around(|storage| {
            // let r = factory_book_record(storage, FactoryAction::Insert);
            let url = "https://www.oreilly.com/library/view/".to_string();
            let mut r = BookRecord::new(&url);
            r.insert(&storage).unwrap();
            let books = BookRecord::all(&storage);
            assert_eq!(books, vec![r]);
        })
    }
    #[test]
    fn book_record_delete() {
        around(|storage| {
            let url = "https://www.oreilly.com/library/view/".to_string();
            let mut r = BookRecord::new(&url);
            r.insert(&storage).unwrap();
            let books = BookRecord::all(storage);
            assert_eq!(books, vec![r.clone()]);
            r.delete(storage).unwrap();
            let books = BookRecord::all(storage);
            assert_eq!(books, vec![]);
        })
    }
    #[test]
    fn storage_db_path() {
        let storage = Storage::new(Some("tmp/test.db".to_string()));
        assert_eq!(storage.db_path(), "tmp/test.db".to_string());
        let storage = Storage::new(None);
        assert_eq!(
            storage.db_path(),
            dirs::home_dir()
                .unwrap()
                .join(".oreally/database.db")
                .to_str()
                .unwrap()
                .to_string()
        );
    }
    #[test]
    fn storage_conn() {
        around(|storage| {
            let conn = storage.conn().unwrap();
            assert_eq!(conn.query_row("SELECT 1", [], |row| row.get(0)), Ok(1));
        })
    }
    #[test]
    fn storage_is_ready() {
        around(|storage| {
            assert!(storage.is_ready());
        });
        let storage = Storage::new(Some("tmp/a_db.db".to_string()));
        assert_eq!(storage.is_ready(), false);
    }
    #[test]
    fn storage_setup() {
        around(|storage| assert!(storage.setup().is_err()));
        let storage = Storage::new(Some("tmp/a_db_for_setup.db".to_string()));
        storage.drop().unwrap();
        assert!(storage.setup().is_ok());
        storage.drop().unwrap();
    }
}
