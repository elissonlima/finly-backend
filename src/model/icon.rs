use serde::Serialize;

#[derive(Serialize)]
pub struct XmlIcon {
    pub id: i32,
    pub name: String,
    pub xml: String
}
