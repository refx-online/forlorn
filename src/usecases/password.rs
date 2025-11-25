use anyhow::Result;

pub async fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let is_valid = bcrypt::verify(password, hash)?;

    Ok(is_valid)
}
