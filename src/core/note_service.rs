//! Note lifecycle helpers, independent from storage and UI.

use uuid::Uuid;

use super::models::Note;

/// Create a blank note, insert it at the top and return its identifier.
pub fn create_note(notes: &mut Vec<Note>) -> Uuid {
    let mut note = Note::new();
    note.touch();
    let id = note.id;
    notes.insert(0, note);
    id
}

/// Remove a note. Returns the id that should become active, or `None` when the list is empty.
pub fn delete_note(notes: &mut Vec<Note>, id: Uuid) -> Option<Uuid> {
    if let Some(index) = notes.iter().position(|n| n.id == id) {
        notes.remove(index);
        return notes
            .get(index.min(notes.len().saturating_sub(1)))
            .map(|n| n.id);
    }
    None
}

/// Sort by last modification, most recent first.
pub fn sort_notes(notes: &mut [Note]) {
    notes.sort_by(|a, b| {
        b.updated_at
            .cmp(&a.updated_at)
            .then_with(|| b.id.cmp(&a.id))
    });
}

/// Ensure the library always contains at least one note.
pub fn ensure_non_empty(notes: &mut Vec<Note>) -> Uuid {
    if let Some(id) = notes.first().map(|n| n.id) {
        return id;
    }
    create_note(notes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_delete_returns_next_active() {
        let mut notes = Vec::new();
        let a = create_note(&mut notes);
        let b = create_note(&mut notes);
        let c = create_note(&mut notes);
        let next = delete_note(&mut notes, b).expect("next");
        assert!(next == a || next == c);
        assert_eq!(notes.len(), 2);
    }

    #[test]
    fn deleting_the_only_note_leaves_none() {
        let mut notes = Vec::new();
        let a = create_note(&mut notes);
        assert_eq!(delete_note(&mut notes, a), None);
        assert!(notes.is_empty());
    }

    #[test]
    fn ensure_non_empty_creates_a_note() {
        let mut notes = Vec::new();
        let id = ensure_non_empty(&mut notes);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, id);
    }
}
