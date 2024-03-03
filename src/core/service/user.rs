use std::collections::HashSet;

use sea_orm::DatabaseConnection;

use crate::core::data::permission::bulk_get_permission_by_id;

pub async fn get_user_permissions(
    db: &DatabaseConnection,
    user_id: i32,
) -> Result<Vec<String>, sea_orm::DbErr> {
    let user_permissions = crate::core::data::permission::get_user_permissions(db, user_id).await?;

    let user_permission_ids: HashSet<i32> = user_permissions
        .clone()
        .into_iter()
        .map(|permission| permission.id)
        .collect();
    let unique_permission_ids: Vec<i32> = user_permission_ids.into_iter().collect();

    let permissions = bulk_get_permission_by_id(db, unique_permission_ids).await?;

    let mut user_permissions: Vec<String> = Vec::new();

    for permission in permissions {
        let perm = format!("{}.{}", permission.module, permission.name);

        user_permissions.push(perm)
    }

    Ok(user_permissions)
}
