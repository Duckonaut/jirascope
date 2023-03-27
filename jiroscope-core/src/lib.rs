use serde::{Deserialize, Serialize};
use ureq::post;

mod error;

pub use error::Error;

// Empty struct for now. Will almost certainly be expanded later to hold state like tokens and
// cached data.
pub struct Jiroscope;

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: Option<usize>,
    pub message: String,
}

impl Jiroscope {
    pub const fn new() -> Jiroscope {
        Jiroscope
    }

    pub fn register_note(&self, message: String) -> Result<Note, crate::Error> {
        let note = Note { id: None, message };

        let response = post("http://localhost:1937/notes").send_json(note)?;

        let note: Note = response.into_json()?;

        Ok(note)
    }

    pub fn get_notes(&self) -> Result<Vec<Note>, crate::Error> {
        let response = ureq::get("http://localhost:1937/notes").call()?;

        let notes: Vec<Note> = response.into_json()?;

        Ok(notes)
    }

    pub fn get_note_by_id(&self, id: usize) -> Result<Note, crate::Error> {
        let response = ureq::get(&format!("http://localhost:1937/notes/{}", id)).call()?;

        let note: Note = response.into_json()?;

        Ok(note)
    }

    pub fn update_note_by_id(&self, id: usize, message: String) -> Result<Note, crate::Error> {
        let note = Note {
            id: Some(id),
            message,
        };

        let response = ureq::put(&format!("http://localhost:1937/notes/{}", id)).send_json(note)?;

        let note: Note = response.into_json()?;

        Ok(note)
    }
}

impl Default for Jiroscope {
    fn default() -> Self {
        Self::new()
    }
}
