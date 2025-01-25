use std::sync::Arc;

use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    IndexModel,
};

use crate::{
    controllers::{
        file_controller::file_controller_controller::create_file_image,
        school_controller::{
            sector_controller::get_sector_by_id, trade_controller::get_trade_by_id,
        },
        user_controller::user_controller_controller::{
            controller_get_user_by_id, controller_user_get_user_by_email,
        },
    },
    error::db_class_error::{DbClassError, DbClassResult},
    libs::functions::{
        characters_fn::{generate_code, generate_username, is_valid_email, is_valid_username},
        resources::check_if_exit::{
            check_sector_trade_exit, CheckSectorTradeExitModel, UsernameValidator,
        },
    },
    models::class_model::class_model_model::{
        ClassModel, ClassModelGet, ClassModelNew, ClassModelPut,
    },
    AppState,
};

use super::{
    class_room_controller::get_class_room_by_id, class_type_controller::get_class_type_by_id,
};
async fn get_other_collection(
    state: Arc<AppState>,
    class: ClassModel,
) -> DbClassResult<ClassModelGet> {
    let mut formatted_class = ClassModel::format(class.clone());

    if let Some(ref trade_id) = class.trade_id {
        let trade = get_trade_by_id(state.clone(), *trade_id).await?;
        formatted_class.trade = trade.username.or(Some(trade.name));
    }

    if let Some(ref sector_id) = class.sector_id {
        let sector = get_sector_by_id(state.clone(), *sector_id).await?;
        formatted_class.sector = sector.username.or(Some(sector.name));
    }

    if let Some(ref class_type_id) = class.class_type_id {
        let document = get_class_type_by_id(state.clone(), *class_type_id).await?;
        formatted_class.class_type = document.username.or(Some(document.name));
    }

    if let Some(ref class_teacher_id) = class.class_teacher_id {
        let document = controller_get_user_by_id(state.clone(), *class_teacher_id)
            .await
            .map_err(|e| DbClassError::OtherError { err: e.to_string() })?;
        formatted_class.class_teacher = document.username.or(Some(document.name));
    }

    if let Some(class_room_id) = class.class_room_id {
        let document = get_class_room_by_id(state.clone(), class_room_id)
            .await
            .map_err(|e| DbClassError::OtherError { err: e.to_string() })?;
        formatted_class.class_room = document.username.or(Some(document.name));
    }

    // if let Some(symbol_id) = class.symbol_id {
    //     let get_symbol = get_file_by_id(state.clone(), symbol_id).await?;
    //     format_class_room.symbol = Some(get_symbol.src);
    // };

    Ok(formatted_class)
}

async fn validate_class_username(
    state: Arc<AppState>,
    username: &str,
    id_to_exclude: Option<ObjectId>,
) -> DbClassResult<()> {
    let validator = UsernameValidator::new(state.clone());

    validator
        .validate(username, id_to_exclude, move |state, username| {
            let username = username.to_string();
            Box::pin(async move {
                let class = get_class_by_username(state, &username.to_string()).await;
                class.map(|class| Some(class.id)).or_else(|err| {
                    if matches!(err, DbClassError::OtherError { .. }) {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                })
            })
        })
        .await
}

pub async fn create_class(
    state: Arc<AppState>,
    mut class: ClassModelNew,
) -> DbClassResult<ClassModelGet> {
    if let Some(ref username) = class.username {
        let _ = validate_class_username(state.clone(), username, None).await;
    } else {
        return Err(DbClassError::OtherError {
            err: "Username is missing".to_string(),
        });
    }
    check_sector_trade_exit(
        state.clone(),
        CheckSectorTradeExitModel {
            sector: class.sector.clone(),
            trade: class.trade.clone(),
        },
    )
    .await?;

    if let Some(ref user_email) = class.class_teacher {
        is_valid_email(user_email).map_err(|e| DbClassError::OtherError { err: e })?;

        let teacher = controller_user_get_user_by_email(state.clone(), user_email.clone())
            .await
            .map_err(|e| DbClassError::OtherError { err: e.to_string() })?;

        if let Some(teacher_role) = teacher.role {
            if teacher_role != "Student" {
                class.class_teacher = Some(teacher.id);
            } else {
                return Err(DbClassError::OtherError {
                    err: format!(
                        "This user is not allowed [{}] to create class, because his/her role is student [{}]",
                        teacher.name, teacher_role
                    ),
                });
            }
        } else {
            return Err(DbClassError::OtherError {
                err: format!(
                    "This user is not allowed [{}] to create class, because his/he don't have role",
                    teacher.name,
                ),
            });
        }
    }

    if class.username.is_none() {
        class.username = Some(generate_username(&class.name));
    }

    if class.code.is_none() {
        class.code = Some(generate_code());
    }

    let index = IndexModel::builder()
        .keys(doc! {"username": 1, "code": 1})
        .options(IndexOptions::builder().unique(true).build())
        .build();

    state
        .db
        .class
        .collection
        .create_index(index)
        .await
        .map_err(|e| DbClassError::OtherError { err: e.to_string() })?;

    if let Some(file) = class.symbol {
        let symbol =
            create_file_image(state.clone(), file, "class room symbol".to_string()).await?;
        class.symbol = Some(symbol);
    }

    let create = state
        .db
        .class
        .create(ClassModel::new(class), Some("class".to_string()))
        .await?;

    get_class_by_id(state, create).await
}

pub async fn get_class_by_username(
    state: Arc<AppState>,
    username: &String,
) -> DbClassResult<ClassModelGet> {
    let get = state
        .db
        .class
        .collection
        .find_one(doc! {"username": &username})
        .await?
        .ok_or(DbClassError::OtherError {
            err: format!("Class not found by username [{}]", &username),
        })?;

    get_other_collection(state, get).await
}

pub async fn get_class_by_id(state: Arc<AppState>, id: ObjectId) -> DbClassResult<ClassModelGet> {
    let get = state
        .db
        .class
        .get_one_by_id(id, Some("class".to_string()))
        .await?;

    get_other_collection(state, get).await
}

pub async fn get_all_class(state: Arc<AppState>) -> DbClassResult<Vec<ClassModelGet>> {
    let get = state
        .db
        .class
        .get_many(None, Some("class".to_string()))
        .await?;

    let mut class_gets = Vec::new();

    for class in get {
        let my_class = get_other_collection(state.clone(), class).await?;
        class_gets.push(my_class);
    }

    Ok(class_gets)
}

pub async fn update_class_by_id(
    state: Arc<AppState>,
    id: ObjectId,
    mut class: ClassModelPut,
) -> DbClassResult<ClassModelGet> {
    check_sector_trade_exit(
        state.clone(),
        CheckSectorTradeExitModel {
            sector: class.sector.clone(),
            trade: class.trade.clone(),
        },
    )
    .await?;

    if let Some(ref user_email) = class.class_teacher {
        is_valid_email(user_email).map_err(|e| DbClassError::OtherError { err: e })?;

        let teacher = controller_user_get_user_by_email(state.clone(), user_email.clone())
            .await
            .map_err(|e| DbClassError::OtherError { err: e.to_string() })?;

        if let Some(teacher_role) = teacher.role {
            if teacher_role != "Student" {
                class.class_teacher = Some(teacher.id);
            } else {
                return Err(DbClassError::OtherError {
                    err: format!(
                        "This user is not allowed [{}] to create class, because his/her role is student [{}]",
                        teacher.name, teacher_role
                    ),
                });
            }
        } else {
            return Err(DbClassError::OtherError {
                err: format!(
                    "This user is not allowed [{}] to create class, because his/he don't have role",
                    teacher.name,
                ),
            });
        }
    }

    if let Some(ref username) = class.username {
        is_valid_username(username).map_err(|err| DbClassError::OtherError { err })?;
        let get_username = get_class_by_username(state.clone(), username).await;
        if get_username.is_ok() {
            return Err(DbClassError::OtherError {
                err: format!(
                    "Username sector already exists [{}], please try another",
                    username
                ),
            });
        }
    }

    state
        .db
        .class
        .update(id, ClassModel::put(class), Some("class".to_string()))
        .await?;
    get_class_by_id(state, id).await
}

pub async fn delete_class_by_id(
    state: Arc<AppState>,
    id: ObjectId,
) -> DbClassResult<ClassModelGet> {
    let get = get_class_by_id(state.clone(), id).await?;
    state.db.class.delete(id, Some("class".to_string())).await?;
    Ok(get)
}
