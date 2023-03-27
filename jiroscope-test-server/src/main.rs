use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::sync::Mutex;

static mut NOTES: Lazy<Mutex<Vec<Note>>> = Lazy::new(|| Mutex::new(Vec::new())); // Needs to be
                                                                                 // lazy because of
                                                                                 // the tokio::sync::Mutex

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /notes` goes to `create_note`
        .route("/notes", post(create_note))
        // `GET /notes` goes to `get_notes`
        .route("/notes", get(get_notes))
        // `GET /notes/{id}` goes to `get_note_by_id`
        .route("/notes/:id", get(get_note_by_id))
        // POST /notes/{id} goes to `update_note_by_id`
        .route("/notes/:id", post(update_note_by_id));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 1937));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Nothing to see here"
}

async fn create_note(Json(note): Json<Note>) -> (StatusCode, Json<Option<Note>>) {
    if note.id.is_some() {
        (StatusCode::BAD_REQUEST, Json(None))
    } else {
        let mut notes = unsafe { NOTES.lock().await };
        let id = notes.len();
        let note = Note {
            id: Some(id),
            message: note.message,
        };
        notes.push(note.clone());

        (StatusCode::CREATED, Json(Some(note)))
    }
}

async fn get_notes() -> Json<Vec<Note>> {
    let notes = unsafe { NOTES.lock().await };
    Json(notes.clone())
}

async fn get_note_by_id(Path(id): Path<usize>) -> (StatusCode, Json<Option<Note>>) {
    let notes = unsafe { NOTES.lock().await };
    let note = notes.get(id).cloned();

    match note {
        Some(note) => (StatusCode::OK, Json(Some(note))),
        None => (StatusCode::NOT_FOUND, Json(None)),
    }
}

async fn update_note_by_id(
    Path(id): Path<usize>,
    Json(note): Json<Note>,
) -> (StatusCode, Json<Option<Note>>) {
    let mut notes = unsafe { NOTES.lock().await };
    let current_note = notes.get_mut(id);

    match current_note {
        Some(current_note) => {
            current_note.message = note.message.clone();
            (StatusCode::OK, Json(Some(note.clone())))
        }
        None => (StatusCode::NOT_FOUND, Json(None)),
    }
}

// the output to our `create_user` handler
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Note {
    id: Option<usize>,
    message: String,
}
