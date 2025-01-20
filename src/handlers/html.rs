use std::path::PathBuf;

use actix_files::NamedFile;

pub async fn terms_of_use() -> actix_web::Result<NamedFile> {
    let path: PathBuf = "/app/html/terms.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}
