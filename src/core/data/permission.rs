use entity::auth_permission::Model as Permission;
use entity::auth_user_permission::Model as UserPermission;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

pub async fn create_permission(
    db: &DatabaseConnection,
    module: &str,
    name: &str,
    hidden: bool,
) -> Result<Permission, sea_orm::DbErr> {
    match get_permission_by_name(db, module, name).await? {
        Some(permission) => Ok(permission),
        None => {
            let permission = entity::auth_permission::ActiveModel {
                module: Set(module.to_string()),
                name: Set(name.to_string()),
                hidden: Set(hidden),
                ..Default::default()
            };

            permission.insert(db).await
        }
    }
}

pub async fn create_user_permission(
    db: &DatabaseConnection,
    permission_id: i32,
    user_id: i32,
) -> Result<UserPermission, sea_orm::DbErr> {
    let user_permission = entity::auth_user_permission::ActiveModel {
        user_id: Set(user_id),
        permission_id: Set(permission_id),
        ..Default::default()
    };

    user_permission.insert(db).await
}

pub async fn get_permission_by_name(
    db: &DatabaseConnection,
    module: &str,
    name: &str,
) -> Result<Option<Permission>, sea_orm::DbErr> {
    entity::prelude::AuthPermission::find()
        .filter(entity::auth_permission::Column::Module.eq(module))
        .filter(entity::auth_permission::Column::Name.eq(name))
        .one(db)
        .await
}

pub async fn bulk_get_permission_by_id(
    db: &DatabaseConnection,
    permissions_ids: Vec<i32>,
) -> Result<Vec<Permission>, sea_orm::DbErr> {
    entity::prelude::AuthPermission::find()
        .filter(entity::auth_permission::Column::Id.is_in(permissions_ids))
        .all(db)
        .await
}

pub async fn get_users_with_permission(
    db: &DatabaseConnection,
    module: &str,
    name: &str,
) -> Result<Vec<UserPermission>, sea_orm::DbErr> {
    let permission_id = match get_permission_by_name(db, module, name).await? {
        Some(permission) => permission.id,
        None => return Ok(Vec::<UserPermission>::new()),
    };

    entity::prelude::AuthUserPermission::find()
        .filter(entity::auth_user_permission::Column::PermissionId.eq(permission_id))
        .all(db)
        .await
}

pub async fn get_user_permissions(
    db: &DatabaseConnection,
    user_id: i32,
) -> Result<Vec<UserPermission>, sea_orm::DbErr> {
    entity::prelude::AuthUserPermission::find()
        .filter(entity::auth_user_permission::Column::UserId.eq(user_id))
        .all(db)
        .await
}
