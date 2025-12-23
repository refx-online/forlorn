pub struct ScreenshotUpload {
    pub username: String,
    pub password_md5: String,
    pub version: Option<i32>,
    pub screenshot_data: Vec<u8>,
}
