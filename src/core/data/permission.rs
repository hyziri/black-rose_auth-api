use entity::auth_permission::Model as Permission;
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
