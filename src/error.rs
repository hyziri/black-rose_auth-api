use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbOrReqwestError {
    #[error(transparent)]
    DbError(#[from] sea_orm::DbErr),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}
