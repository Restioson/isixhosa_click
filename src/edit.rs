use crate::details::{word, WordChangeMethod};
use crate::submit::{
    edit_word_page, qs_form, submit_suggestion, suggest_word_deletion, WordId, WordSubmission,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use warp::{body, Filter, Rejection, Reply};

pub fn edit(
    db: Pool<SqliteConnectionManager>,
) -> impl Filter<Error = Rejection, Extract: Reply> + Clone {
    let db = warp::any().map(move || db.clone());

    let submit_page = warp::get()
        .and(db.clone())
        .and(warp::any().map(|| None)) // previous_success is none
        .and(warp::path!["word" / u64 / "edit"])
        .and(warp::path::end())
        .and_then(edit_word_page);

    let submit_form = warp::post()
        .and(warp::path!["word" / u64])
        .and(warp::path::end())
        .and(body::content_length_limit(4 * 1024))
        .and(qs_form())
        .and(db.clone())
        .and_then(submit_suggestion_reply);

    let failed_to_submit = warp::any()
        .and(db.clone())
        .and(warp::any().map(|| Some(false))) // previous_success is Some(false)
        .and(warp::path!["word" / u64])
        .and(warp::path::end())
        .and_then(edit_word_page);

    let delete_redirect = warp::post()
        .and(warp::path!["word" / u64 / "delete"])
        .and(warp::path::end())
        .and(db)
        .and_then(delete_word_reply);

    submit_page
        .or(submit_form)
        .or(delete_redirect)
        .or(failed_to_submit)
}

async fn submit_suggestion_reply(
    id: u64,
    w: WordSubmission,
    db: Pool<SqliteConnectionManager>,
) -> Result<impl Reply, Rejection> {
    submit_suggestion(w, &db).await;
    word(id, db, Some(WordChangeMethod::Edit)).await
}

async fn delete_word_reply(
    id: u64,
    db: Pool<SqliteConnectionManager>,
) -> Result<impl Reply, Rejection> {
    suggest_word_deletion(WordId(id), &db).await;
    word(id, db, Some(WordChangeMethod::Delete)).await
}
