use anyhow::Result;

use crate::{infrastructure::database::DbPoolManager, models::Favourites};

pub async fn fetch_all(db: &DbPoolManager, userid: i32) -> Result<Option<Favourites>> {
    let favourites = sqlx::query_as::<_, Favourites>("select * from favourites where userid = ?")
        .bind(userid)
        .fetch_optional(db.as_ref())
        .await?;

    Ok(favourites)
}

pub async fn fetch_one(db: &DbPoolManager, userid: i32, setid: i32) -> Result<Option<Favourites>> {
    let favourite =
        sqlx::query_as::<_, Favourites>("select * from favourites where userid = ? and setid = ?")
            .bind(userid)
            .bind(setid)
            .fetch_optional(db.as_ref())
            .await?;

    Ok(favourite)
}

pub async fn insert(db: &DbPoolManager, userid: i32, setid: i32) -> Result<()> {
    sqlx::query(
        "insert into favourites (userid, setid, created_at) values (?, ?, UNIX_TIMESTAMP())",
    )
    .bind(userid)
    .bind(setid)
    .execute(db.as_ref())
    .await?;

    Ok(())
}
