use std::{
    env,
    fs::{self, File},
    path::Path,
};

use rusqlite::{Connection, Result};

use crate::BookRequest;

#[derive(Debug)]
pub struct BookRecord {
    id: Option<i32>,
    pub url: String,
}

pub struct Storage {}

impl Storage {
    pub fn db_path() -> String {
        // TODO: Use a better way to get home directory
        let home = env::home_dir().unwrap();
        let db_path = home.join(".oreally/oreilly.db");
        db_path.to_str().unwrap().to_string()
    }

    pub fn conn() -> Result<Connection> {
        let db_path = Storage::db_path();
        let conn = Connection::open(db_path)?;
        Ok(conn)
    }

    pub fn setup() -> Result<(), Box<dyn std::error::Error>> {
        let db_path = Storage::db_path();
        let folder = Path::new(&db_path).parent().unwrap();
        fs::create_dir_all(folder).unwrap_or_else(|_| {
            panic!("unable to create folder {folder:?} for database file");
        });
        File::create(&db_path)
            .unwrap_or_else(|_| panic!("unable to create database file at {db_path:?}"));
        let conn = Self::conn()?;
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
    pub fn insert(&mut self) -> Result<&mut Self, Box<dyn std::error::Error>> {
        if self.id.is_some() {
            return Ok(self);
        }
        let conn = Storage::conn()?;
        let id = conn.execute(
            "INSERT INTO book_queue (url) VALUES (?1) RETURNING id",
            [&self.url],
        )?;
        self.id = Some(id.try_into()?);
        Ok(self)
    }
    pub fn all() -> Result<Vec<BookRecord>, Box<dyn std::error::Error>> {
        let conn = Storage::conn()?;
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

    pub fn delete(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.id.is_none() {
            return Ok(());
        }
        let conn = Storage::conn()?;
        conn.execute("DELETE FROM book_queue WHERE id = ?1", [self.id])?;
        Ok(())
    }
}

// pub fn add_book(BookRecord { id: _, url }: BookRecord) -> Result<(), Box<dyn std::error::Error>> {
//     let conn = Connection::open("~/.oreally/oreilly.db")?;
//     conn.execute("INSERT INTO book_queue (url) VALUES (?1)", [url])?;
//     Ok(())
// }

// pub fn get_pending() -> Result<Vec<BookRecord>, Box<dyn std::error::Error>> {
//     let conn = Connection::open("~/.oreally/oreilly.db")?;
//     let mut stmt = conn.prepare("SELECT id, url FROM book_queue")?;
//     let book_iter = stmt.query_map([], |row| {
//         Ok(BookRecord {
//             id: row.get(0)?,
//             url: row.get(1)?,
//         })
//     })?;

//     let mut books = Vec::new();
//     for book in book_iter {
//         books.push(book.unwrap());
//     }
//     Ok(books)
// }

// pub fn remove_book(id: i32) -> Result<(), Box<dyn std::error::Error>> {
//     let conn = Connection::open("~/.oreally/oreilly.db")?;
//     conn.execute("DELETE FROM book WHERE id = ?1", [id])?;
//     Ok(())
// }

// fn main() -> Result<()> {
//     let conn = Connection::open_in_memory()?;

//     conn.execute(
//         "CREATE TABLE person (
//             id    INTEGER PRIMARY KEY,
//             name  TEXT NOT NULL,
//             data  BLOB
//         )",
//         (), // empty list of parameters.
//     )?;
//     let me = Person {
//         id: 0,
//         name: "Steven".to_string(),
//         data: None,
//     };
//     conn.execute(
//         "INSERT INTO person (name, data) VALUES (?1, ?2)",
//         (&me.name, &me.data),
//     )?;

//     let mut stmt = conn.prepare("SELECT id, name, data FROM person")?;
//     let person_iter = stmt.query_map([], |row| {
//         Ok(Person {
//             id: row.get(0)?,
//             name: row.get(1)?,
//             data: row.get(2)?,
//         })
//     })?;

//     for person in person_iter {
//         println!("Found person {:?}", person.unwrap());
//     }
//     Ok(())
// }
