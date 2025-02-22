use actix_files::Directory;
use actix_web::{dev::ServiceResponse, HttpRequest, HttpResponse};
use regex::Regex;
use serde_json::json;

pub fn file_list_handler(
    dir: &Directory,
    req: &HttpRequest,
) -> Result<ServiceResponse, std::io::Error> {
    let mut files: Vec<String> = dir
        .path
        .read_dir()
        .unwrap()
        .filter_map(|d| match d {
            Ok(dir) => {
                let name = dir.file_name();
                let s = name.to_string_lossy().to_string();
                if s.ends_with(".svg") {
                    Some(s)
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .collect();
    files.sort_by(|a, b| {
        let default_res = a.cmp(b);
        let re: Regex = match Regex::new("^[0-9]+") {
            Ok(r) => r,
            Err(_) => return default_res,
        };
        let prefix_a_str = match re.find(a.as_str()) {
            Some(s) => s.as_str(),
            None => return default_res,
        };
        let prefix_b_str = match re.find(b.as_str()) {
            Some(s) => s.as_str(),
            None => return default_res,
        };

        let prefix_a_int = match prefix_a_str.parse::<i32>() {
            Ok(d) => d,
            Err(_) => return default_res,
        };
        let prefix_b_int = match prefix_b_str.parse::<i32>() {
            Ok(d) => d,
            Err(_) => return default_res,
        };

        prefix_a_int.cmp(&prefix_b_int)
    });
    Ok(ServiceResponse::new(
        req.clone(),
        HttpResponse::Ok().body(json!(files).to_string()),
    ))
}
