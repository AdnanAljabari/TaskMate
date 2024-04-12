use rocket::{get, post, delete, routes, Route};
use rocket_contrib::json::Json;
use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use csv::{Reader, Writer};


#[derive(Debug, Serialize, Deserialize)]
struct Todo {
    title: String,
    done: bool,
}


struct Database {
    file_path: String,
}

impl Database {
    fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    fn read_todos(&self) -> Result<Vec<Todo>, io::Error> {
        let file = File::open(&self.file_path)?;
        let mut rdr = Reader::from_reader(file);
        let mut todos = Vec::new();
        for result in rdr.deserialize() {
            let todo: Todo = result?;
            todos.push(todo);
        }
        Ok(todos)
    }

    fn write_todos(&self, todos: &[Todo]) -> Result<(), io::Error> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.file_path)?;
        let mut wtr = Writer::from_writer(file);
        for todo in todos {
            wtr.serialize(todo)?;
        }
        wtr.flush()?;
        Ok(())
    }

    fn add_todo(&self, new_todo: Todo) -> Result<(), io::Error> {
        let mut todos = self.read_todos()?;
        todos.push(new_todo);
        self.write_todos(&todos)
    }

    fn delete_todo(&self, index: usize) -> Result<(), io::Error> {
        let mut todos = self.read_todos()?;
        if index < todos.len() {
            todos.remove(index);
            self.write_todos(&todos)?;
        }
        Ok(())
    }
}

#[get("/todos")]
fn list_todos(db: rocket::State<Database>) -> Json<Vec<Todo>> {
    let todos = db.read_todos().unwrap_or_else(|_| vec![]);
    Json(todos)
}

#[post("/todos", data = "<new_todo>")]
fn add_todo(new_todo: Json<Todo>, db: rocket::State<Database>) {
    db.add_todo(new_todo.into_inner()).unwrap();
}

#[delete("/todos/<index>")]
fn delete_todo(index: usize, db: rocket::State<Database>) {
    db.delete_todo(index).unwrap();
}

#[rocket::main]
async fn main() {
    rocket::build()
        .manage(Database::new("todos.csv"))
        .mount("/", routes![list_todos, add_todo, delete_todo])
        .launch()
        .await
        .unwrap();
}
