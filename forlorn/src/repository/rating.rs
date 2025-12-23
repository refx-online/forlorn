use anyhow::Result;

use crate::infrastructure::database::DbPoolManager;

pub async fn fetch_average_rating(db: &DbPoolManager, map_md5: &str) -> Result<f32> {
    let avg: Option<f32> = sqlx::query_scalar("select avg(rating) from ratings where map_md5 = ?")
        .bind(map_md5)
        .fetch_optional(db.as_ref())
        .await?
        .flatten();

    Ok(avg.unwrap_or(0.0))
}

pub async fn fetch(db: &DbPoolManager, map_md5: &str, user_id: i32) -> Result<Option<u8>> {
    let rating: Option<u8> =
        sqlx::query_scalar("select rating from ratings where map_md5 = ? and userid = ?")
            .bind(map_md5)
            .bind(user_id)
            .fetch_optional(db.as_ref())
            .await?
            .flatten();

    Ok(rating)
}

pub async fn fetch_many(db: &DbPoolManager, map_md5: &str) -> Result<Vec<(i32, u8)>> {
    let ratings: Vec<(i32, u8)> =
        sqlx::query_as::<_, (i32, u8)>("select userid, rating from ratings where map_md5 = ?")
            .bind(map_md5)
            .fetch_all(db.as_ref())
            .await?;

    Ok(ratings)
}

pub async fn insert(db: &DbPoolManager, map_md5: &str, user_id: i32, rating: u8) -> Result<()> {
    sqlx::query("insert into ratings (map_md5, userid, rating) values (?, ?, ?)")
        .bind(map_md5)
        .bind(user_id)
        .bind(rating)
        .execute(db.as_ref())
        .await?;

    Ok(())
}
