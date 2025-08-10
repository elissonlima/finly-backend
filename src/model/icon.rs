use serde::Serialize;

#[derive(Serialize)]
pub struct XmlIcon {
    pub id: i32,
    pub xml: String
}