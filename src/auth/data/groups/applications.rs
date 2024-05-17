use anyhow::anyhow;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DeleteResult, EntityTrait,
    QueryFilter,
};
use std::collections::{HashMap, HashSet};

use crate::{
    auth::model::groups::GroupApplicationDto, eve::service::affiliation::get_character_affiliations,
};

use entity::sea_orm_active_enums::{GroupApplicationStatus, GroupApplicationType, GroupType};

use super::get_group_by_id;

use crate::auth::data::user::{bulk_get_user_main_characters, get_user};
use entity::auth_group_application::Model as GroupApplication;

pub async fn get_group_application(
    db: &DatabaseConnection,
    application_status: Option<GroupApplicationStatus>,
    application_type: Option<GroupApplicationType>,
    application_id: Option<i32>,
    group_id: Option<i32>,
    user_id: Option<i32>,
) -> Result<Vec<GroupApplicationDto>, anyhow::Error> {
    if let Some(group_id) = group_id {
        match get_group_by_id(db, group_id).await? {
            Some(group) => {
                if group.group_type == GroupType::Open || group.group_type == GroupType::Auto {
                    return Err(anyhow!("Group does not require applications"));
                }
            }
            None => return Err(anyhow!("Group does not exist")),
        };
    };

    if let Some(user_id) = user_id {
        match get_user(db, user_id).await? {
            Some(_) => (),
            None => return Err(anyhow!("User does not exist")),
        };
    };

    let mut query = entity::prelude::AuthGroupApplication::find();

    if let Some(application_type) = application_type {
        query = query
            .filter(entity::auth_group_application::Column::RequestType.eq(Some(application_type)));
    }

    if let Some(application_status) = application_status {
        query = query
            .filter(entity::auth_group_application::Column::Status.eq(Some(application_status)));
    }

    if let Some(application_id) = application_id {
        query = query.filter(entity::auth_group_application::Column::Id.eq(Some(application_id)));
    }

    if let Some(group_id) = group_id {
        query = query.filter(entity::auth_group_application::Column::GroupId.eq(Some(group_id)));
    }

    if let Some(user_id) = user_id {
        query = query.filter(entity::auth_group_application::Column::UserId.eq(Some(user_id)));
    }

    let applications = query.all(db).await?;

    let user_ids: HashSet<i32> = applications
        .iter()
        .flat_map(|app| {
            let mut ids = vec![app.user_id];
            if let Some(responder_id) = app.responder {
                ids.push(responder_id);
            }
            ids.into_iter()
        })
        .collect();
    let user_ids: Vec<i32> = user_ids.into_iter().collect();

    let mains = bulk_get_user_main_characters(db, user_ids).await?;

    let character_ids = mains
        .iter()
        .map(|main| main.character_id)
        .collect::<Vec<i32>>();
    let affiliations = get_character_affiliations(db, character_ids).await?;

    let mut applications_map: HashMap<i32, _> = applications
        .into_iter()
        .map(|app| (app.user_id, app))
        .collect();

    let mut group_applications = vec![];

    for main in mains.clone() {
        if let (Some(character), Some(application)) = (
            affiliations
                .iter()
                .find(|affiliation| affiliation.character_id == main.character_id),
            applications_map.remove(&main.user_id),
        ) {
            let mut responder_info = None;

            if let Some(responder) = application.responder {
                let main_character = mains
                    .iter()
                    .find(|main| main.user_id == responder)
                    .map(|main| main.character_id);

                if let Some(main_character) = main_character {
                    let responder_character = affiliations
                        .iter()
                        .find(|affiliation| affiliation.character_id == main_character);

                    if let Some(responder_character) = responder_character {
                        responder_info = Some(responder_character.clone());
                    }
                }
            };

            let group_application = GroupApplicationDto {
                id: application.id,
                group_id: application.group_id,
                user_id: application.user_id,
                applicant_info: character.clone(),
                responder_info,
                status: application.status.into(),
                request_type: application.request_type.into(),
                request_message: application.request_message,
                response_message: application.response_message,
                created: DateTime::from_naive_utc_and_offset(application.created, Utc),
                last_updated: DateTime::from_naive_utc_and_offset(application.last_updated, Utc),
            };

            group_applications.push(group_application);
        }
    }

    Ok(group_applications)
}

pub async fn update_group_application(
    db: &DatabaseConnection,
    application_id: i32,
    application_request_message: Option<String>,
    application_response_message: Option<String>,
    application_status: Option<GroupApplicationStatus>,
    application_responder: Option<i32>,
) -> Result<GroupApplication, anyhow::Error> {
    let application = entity::prelude::AuthGroupApplication::find()
        .filter(entity::auth_group_application::Column::Id.eq(application_id))
        .one(db)
        .await?;

    match application {
        Some(application) => {
            if application.status != GroupApplicationStatus::Outstanding {
                return Err(anyhow!("Not allowed to update a completed application"));
            }

            let mut application: entity::auth_group_application::ActiveModel = application.into();

            if let Some(application_request_message) = application_request_message {
                if application_request_message.is_empty() {
                    application.request_message = Set(None);
                } else {
                    application.request_message = Set(Some(application_request_message));
                }
            }

            if let Some(application_response_message) = application_response_message {
                if application_response_message.is_empty() {
                    application.response_message = Set(None);
                } else {
                    application.request_message = Set(Some(application_response_message));
                }
            }

            if let Some(application_status) = application_status {
                application.status = Set(application_status);
            }

            if let Some(application_responder) = application_responder {
                application.responder = Set(Some(application_responder));
            }

            application.last_updated = Set(Utc::now().naive_utc());

            let application = application.update(db).await?;

            Ok(application)
        }
        None => Err(anyhow!("Application not found")),
    }
}

pub async fn delete_group_application(
    db: &DatabaseConnection,
    application_id: i32,
) -> Result<DeleteResult, anyhow::Error> {
    let application = entity::prelude::AuthGroupApplication::find()
        .filter(entity::auth_group_application::Column::Id.eq(application_id))
        .one(db)
        .await?;

    match application {
        Some(application) => {
            if application.status != GroupApplicationStatus::Outstanding {
                return Err(anyhow!("Not allowed to delete a completed application"));
            }

            let result = entity::prelude::AuthGroupApplication::delete_by_id(application_id)
                .exec(db)
                .await?;

            Ok(result)
        }
        None => Err(anyhow!("Application not found")),
    }
}
