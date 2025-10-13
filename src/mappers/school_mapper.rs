use crate::{domain::school::School, models::school_token_model::SchoolToken};

pub fn to_school_school_token(school: &School) -> Result<SchoolToken, String> {
    let school_db = match &school.database_name {
        Some(db_name) => db_name.clone(),
        None => {
            return Err(
                "❌ School does not have a database_name. Token cannot be created.".to_string(),
            );
        }
    };
    Ok(SchoolToken {
        id: school
            .id
            .as_ref()
            .map(|id| id.to_string())
            .unwrap_or_default(),
        creator_id: Some(
            school
                .creator_id
                .as_ref()
                .map(|id| id.to_string())
                .unwrap_or_default(),
        ),
        name: school.name.clone(),
        username: school.username.clone(),
        logo: school.logo.clone(),
        school_type: school.school_type.clone(),
        affiliation: school.affiliation.clone(),
        database_name: school_db.clone(),
        created_at: school.created_at,
        exp: 0,
        iat: 0,
    })
}
