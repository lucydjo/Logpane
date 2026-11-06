use logpane::core::models::{Note, NoteBlock};
use logpane::core::search::{normalize, search};

fn note(title: &str, body: &str) -> Note {
    let mut note = Note::new().with_title(title);
    note.blocks = vec![NoteBlock::text(body)];
    note
}

#[test]
fn accents_and_case_are_ignored() {
    assert_eq!(normalize("ÉLÈVE à l'École"), "eleve a l'ecole");
    let n = note("Préférence", "Garçon");
    assert_eq!(search(std::slice::from_ref(&n), "preference").len(), 1);
    assert_eq!(search(&[n], "garcon").len(), 1);
}

#[test]
fn title_matches_outrank_body_matches() {
    let title_match = note("Claude vs GPT", "autre chose");
    let body_match = note("Comparaison", "j'ai testé claude vs gpt");
    let hits = search(&[body_match.clone(), title_match.clone()], "claude");
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].note_id, title_match.id);
    assert_eq!(hits[1].note_id, body_match.id);
}

#[test]
fn no_match_returns_empty() {
    let hits = search(&[note("a", "b")], "introuvable");
    assert!(hits.is_empty());
}

#[test]
fn empty_query_lists_everything() {
    let notes = vec![note("a", ""), note("b", ""), note("c", "")];
    assert_eq!(search(&notes, "").len(), 3);
}

#[test]
fn multi_line_body_is_searchable() {
    let n = note("Test", "première ligne\nseconde ligne avec mot-clé");
    assert_eq!(search(&[n], "mot-cle").len(), 1);
}
