// These are functions to serve as placeholders for eve_esi during testing scenarios
// This avoids dependency on eve esi which could cause tests to fail if there are any issues with the API

use chrono::Utc;

use crate::error::DbOrReqwestError;

pub async fn get_alliance(
    _alliance_id: i32,
) -> Result<eve_esi::model::alliance::Alliance, DbOrReqwestError> {
    use eve_esi::model::alliance::Alliance;

    Ok(Alliance {
        creator_corporation_id: 109299958,
        creator_id: 180548812,
        date_founded: Utc::now(),
        executor_corporation_id: Some(109299958),
        name: String::from("C C P Alliance"),
        ticker: "C C P".to_string(),
        faction_id: None,
    })
}

pub async fn get_corporation(
    _corporation_id: i32,
) -> Result<eve_esi::model::corporation::Corporation, DbOrReqwestError> {
    use eve_esi::model::corporation::Corporation;

    Ok(Corporation {
        alliance_id: Some(434243723),
        ceo_id: 180548812,
        creator_id: 180548812,
        date_founded: None,
        description: None,
        faction_id: None,
        home_station_id: None,
        member_count: 20,
        name: String::from("C C P"),
        shares: Some(1000000),
        tax_rate: 10.0,
        ticker: "-CCP-".to_string(),
        url: None,
        war_eligible: None,
    })
}

pub async fn get_character(
    _character_id: i32,
) -> Result<eve_esi::model::character::Character, DbOrReqwestError> {
    use eve_esi::model::character::Character;

    Ok(Character {
        name: "CCP Hellmar".to_string(),
        alliance_id: Some(434243723),
        birthday: Utc::now(),
        bloodline_id: 1,
        corporation_id: 109299958,
        description: None,
        faction_id: None,
        gender: "Male".to_string(),
        race_id: 1,
        security_status: None,
        title: None,
    })
}
