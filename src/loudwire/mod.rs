use chrono::prelude::*;

pub struct Release {
    pub date: DateTime<Local>,
    pub artist: String,
    pub album: String,
}

pub fn get_releases(current_datetime: &DateTime<Local>) -> Result<Vec<Release>, String> {
    let mut release = Vec::new();

    release.push(Release {
        date: *current_datetime,
        artist: String::from("Usnea"),
        album: String::from("Random Album"),
    });

    Ok(release)
}
