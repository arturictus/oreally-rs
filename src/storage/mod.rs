use std::{
    fs::{self, File},
    path::Path,
};

use rusqlite::{Connection, Result};

use crate::BookRequest;

pub fn setup() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = "~/.oreally/oreilly.db";
    let folder = Path::new(db_path).parent().unwrap();
    fs::create_dir_all(folder).unwrap_or_else(|_| {
        panic!("unable to create folder {folder:?} for database file");
    });
    File::create(db_path)
        .unwrap_or_else(|_| panic!("unable to create database file at {db_path:?}"));
    let conn = Connection::open("~/.oreally/oreilly.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS book (
            id    INTEGER PRIMARY KEY,
            title  TEXT NOT NULL,
            auth  TEXT,
            folder  TEXT
        )",
        [],
    )?;
    Ok(())
}

pub fn add_book(opts: BookRequest) -> Result<(), Box<dyn std::error::Error>> {
    let BookRequest {
        id,
        title,
        auth,
        folder,
    } = opts;
    let conn = Connection::open("~/.oreally/oreilly.db")?;
    conn.execute(
        "INSERT INTO book (id, title, auth, folder) VALUES (?1, ?2, ?3, ?4)",
        [id, title, auth, folder],
    )?;
    Ok(())
}

pub fn get_pending() -> Result<Vec<BookRequest>, Box<dyn std::error::Error>> {
    let conn = Connection::open("~/.oreally/oreilly.db")?;
    let mut stmt = conn.prepare("SELECT id, title, auth, folder FROM book")?;
    let book_iter = stmt.query_map([], |row| {
        Ok(BookRequest {
            id: row.get(0)?,
            title: row.get(1)?,
            auth: row.get(2)?,
            folder: row.get(3)?,
        })
    })?;

    let mut books = Vec::new();
    for book in book_iter {
        books.push(book.unwrap());
    }
    Ok(books)
}

pub fn remove_book(id: String) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("~/.oreally/oreilly.db")?;
    conn.execute("DELETE FROM book WHERE id = ?1", [id])?;
    Ok(())
}

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
