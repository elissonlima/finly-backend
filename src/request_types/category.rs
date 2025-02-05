use actix_web::web;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpsertCategoryReq {
    pub id: String,
    pub name: String,
    pub color: String,
    pub icon_name: String,
}

impl From<web::Json<UpsertCategoryReq>> for UpsertCategoryReq {
    fn from(cat: web::Json<UpsertCategoryReq>) -> Self {
        return UpsertCategoryReq {
            id: cat.id.clone(),
            name: cat.name.clone(),
            color: cat.color.clone(),
            icon_name: cat.icon_name.clone(),
        };
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeleteCategoryReq {
    pub category_id: String,
}

impl From<web::Json<DeleteCategoryReq>> for DeleteCategoryReq {
    fn from(cat: web::Json<DeleteCategoryReq>) -> Self {
        return DeleteCategoryReq {
            category_id: cat.category_id.clone(),
        };
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpsertSubcategoryReq {
    pub id: String,
    pub category_id: String,
    pub name: String,
    pub color: String,
    pub icon_name: String,
}

impl From<web::Json<UpsertSubcategoryReq>> for UpsertSubcategoryReq {
    fn from(sub: web::Json<UpsertSubcategoryReq>) -> Self {
        return UpsertSubcategoryReq {
            id: sub.id.clone(),
            category_id: sub.category_id.clone(),
            name: sub.name.clone(),
            color: sub.color.clone(),
            icon_name: sub.icon_name.clone(),
        };
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeleteSubcategoryReq {
    pub subcategory_id: String,
    pub category_id: String,
}

impl From<web::Json<DeleteSubcategoryReq>> for DeleteSubcategoryReq {
    fn from(sub: web::Json<DeleteSubcategoryReq>) -> Self {
        return DeleteSubcategoryReq {
            subcategory_id: sub.subcategory_id.clone(),
            category_id: sub.category_id.clone(),
        };
    }
}
