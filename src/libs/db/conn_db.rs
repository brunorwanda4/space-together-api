use super::{
    class_db::{
        activities_db::ActivityDb, activities_type_db::ActivitiesTypeDb,
        class_group_db::ClassGroupDb,
    },
    conversation_db::message_db::MessageDb,
    request_db::{request_db_db::RequestDb, request_type_db::RequestTypeDb},
    user_db::{user_db_db::UserDb, user_role_db::UserRoleDb},
};
use crate::{
    error::db_error::{DbError, DbResult},
    libs::{
        classes::db_crud::MongoCrud,
        db::{
            conversation_db::conversation_db_db::ConversationDb,
            db_status::db_status_db::get_database_stats,
        },
        schemas::subject_schema::SubjectSchema,
    },
    models::{
        auth::adapter_model::{AccountModel, SessionModel},
        class_model::{
            class_model_model::ClassModel, class_room_model::ClassRoomModel,
            class_room_type_model::ClassRoomTypeModel, class_type_model::ClassTypeModel,
        },
        database_model::collection_model::DatabaseStats,
        education_model::education_model_model::EducationModel,
        file_model::{file_model_model::FileModel, file_type_model::FileTypeModel},
        images_model::{
            profile_images_model::ProfileImageModel, school_logo_model::SchoolLogoModel,
        },
        school_model::{
            school_model_model::SchoolModel, sector_model::SectorModel, trade_model::TradeModel,
        },
        subject_model::{subject_model_model::SubjectModel, subject_type_model::SubjectTypeModel},
    },
};
use dotenv::dotenv;
use mongodb::Client;
use std::env;

#[derive(Debug)]
pub struct ConnDb {
    pub user_role: UserRoleDb,
    pub user: UserDb,
    pub class: MongoCrud<ClassModel>,
    pub class_group: ClassGroupDb,
    pub class_room_type: MongoCrud<ClassRoomTypeModel>,
    pub class_room: MongoCrud<ClassRoomModel>,
    pub conversation: ConversationDb,
    pub message: MessageDb,
    pub activities_type: ActivitiesTypeDb,
    pub activity: ActivityDb,
    pub stats: Option<DatabaseStats>,
    pub request_type: RequestTypeDb,
    pub request: RequestDb,
    pub education: MongoCrud<EducationModel>,
    pub school: MongoCrud<SchoolModel>,
    pub trade: MongoCrud<TradeModel>,
    pub sector: MongoCrud<SectorModel>,
    pub class_type: MongoCrud<ClassTypeModel>,
    pub subject_type: MongoCrud<SubjectTypeModel>,
    pub subject: MongoCrud<SubjectModel>,
    pub subjects: MongoCrud<SubjectSchema>,
    // images
    pub avatars: MongoCrud<ProfileImageModel>,
    pub school_logo: MongoCrud<SchoolLogoModel>,
    // files
    pub file_type: MongoCrud<FileTypeModel>,
    pub file: MongoCrud<FileModel>,
    // auth
    pub session: MongoCrud<SessionModel>,
    pub account: MongoCrud<AccountModel>,
}

impl ConnDb {
    pub async fn init() -> DbResult<Self> {
        dotenv().ok();
        let bd_uri = match env::var("MONGODB_URI") {
            Ok(val) => val.to_string(),
            Err(_) => "mongodb://localhost:27017/".to_string(),
        };

        let client = Client::with_uri_str(bd_uri).await;

        match client {
            Ok(res) => {
                let st_data = res.database("space-together-data-testing");
                let st_image = res.database("space-together-images-testing");

                let stats_result = get_database_stats(&res, "space-together-data").await;
                let stats = match stats_result {
                    Ok(s) => Some(s),
                    Err(_) => None,
                };

                println!("Database connected successfully 🌼");

                Ok(Self {
                    user_role: UserRoleDb {
                        role: st_data.collection("users.role"),
                    },
                    user: UserDb {
                        user: st_data.collection("users"),
                    },
                    class: MongoCrud {
                        collection: st_data.collection("Class"),
                    },
                    class_group: ClassGroupDb {
                        class_group: st_data.collection("--classes_groups"), // private
                    },
                    conversation: ConversationDb {
                        conversation: st_data.collection("--conversations"), // private
                    },
                    message: MessageDb {
                        message: st_data.collection("--messages"), // private collection
                    },
                    activities_type: ActivitiesTypeDb {
                        activities_type: st_data.collection("classes_activities.role"), // role
                    },
                    activity: ActivityDb {
                        activity: st_data.collection("--classes_activities"), //
                    },
                    stats,
                    request_type: RequestTypeDb {
                        request: st_data.collection("requests.role"), // role for request
                    },
                    request: RequestDb {
                        request: st_data.collection("requests"),
                    },
                    education: MongoCrud {
                        collection: st_data.collection("educations"),
                    },
                    school: MongoCrud {
                        collection: st_data.collection("schools"),
                    },
                    sector: MongoCrud {
                        collection: st_data.collection("sector"),
                    },
                    trade: MongoCrud {
                        collection: st_data.collection("trades"),
                    },
                    subject_type: MongoCrud {
                        collection: st_data.collection("subjects.role"),
                    },
                    subject: MongoCrud {
                        collection: st_data.collection("subjects"),
                    },
                    subjects: MongoCrud {
                        collection: st_data.collection("subject"),
                    },
                    class_type: MongoCrud {
                        collection: st_data.collection("classes.role"),
                    },
                    class_room: MongoCrud {
                        collection: st_data.collection("class_room"),
                    },
                    class_room_type: MongoCrud {
                        collection: st_data.collection("class_room.role"),
                    },
                    // images
                    avatars: MongoCrud {
                        collection: st_image.collection("avatars"),
                    },
                    school_logo: MongoCrud {
                        collection: st_image.collection("school_logo"),
                    },
                    // files
                    file_type: MongoCrud {
                        collection: st_data.collection("files.role"),
                    },
                    file: MongoCrud {
                        collection: st_image.collection("files"),
                    },
                    // auth
                    session: MongoCrud {
                        collection: st_data.collection("--session"),
                    },
                    account: MongoCrud {
                        collection: st_data.collection("account"),
                    },
                })
            }
            Err(err) => Err(DbError::CanNotConnectToDatabase {
                err: err.to_string(),
            }),
        }
    }
}
