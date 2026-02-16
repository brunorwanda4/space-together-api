use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};
use futures::TryStreamExt;

use crate::{
    domain::{
        common_details::Paginated,
        role::{Permission, PermissionScope, Role, RolePartial, RoleType, RoleWithRelations, UserRoleAssignment},
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    pipeline::role_pipeline::role_pipeline,
    repositories::base_repo::BaseRepository,
    utils::mongo_utils::{build_search_filter, extract_valid_fields},
};

pub struct RoleService {
    pub collection: Collection<Role>,
    pub assignments_collection: Collection<UserRoleAssignment>,
}

impl RoleService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Role>("roles"),
            assignments_collection: db.collection::<UserRoleAssignment>("user_role_assignments"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::single("name", false),
            IndexDef::single("school_id", false),
            IndexDef::single("role_type", false),
            IndexDef::single("is_active", false),
            IndexDef::compound(vec![("school_id", 1), ("name", 1)], true),
        ];

        let repo = BaseRepository::new(
            self.collection
                .clone()
                .clone_with_type::<mongodb::bson::Document>(),
        );

        repo.ensure_indexes(&indexes).await?;

        let assignment_indexes = vec![
            IndexDef::single("user_id", false),
            IndexDef::single("role_id", false),
            IndexDef::single("school_id", false),
            IndexDef::compound(vec![("user_id", 1), ("school_id", 1)], false),
            IndexDef::compound(vec![("user_id", 1), ("role_id", 1), ("school_id", 1)], true),
        ];

        let assignment_repo = BaseRepository::new(
            self.assignments_collection
                .clone()
                .clone_with_type::<mongodb::bson::Document>(),
        );

        assignment_repo.ensure_indexes(&assignment_indexes).await?;

        Ok(())
    }

    pub async fn seed_default_permissions(&self) -> Result<(), AppError> {
        Ok(())
    }

    pub async fn create_role(&self, dto: Role) -> Result<Role, AppError> {
        self.ensure_indexes().await?;

        if dto.name.trim().is_empty() {
            return Err(AppError {
                message: "Role name cannot be empty".into(),
            });
        }

        let mut filter = doc! { "name": &dto.name };
        if let Some(school_id) = dto.school_id {
            filter.insert("school_id", school_id);
        }

        if self.find_one(None, Some(filter)).await.is_ok() {
            return Err(AppError {
                message: format!("Role with name '{}' already exists", dto.name),
            });
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.create::<Role>(extract_valid_fields(dto.to_document()?), None)
            .await
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<Role, AppError> {
        let mut filter = extra_match.unwrap_or_default();

        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.find_one::<Role>(filter, None)
            .await?
            .ok_or(AppError {
                message: "Role not found".into(),
            })
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<Role>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = ["name", "description", "_id", "school_id", "role_type"];

        let (data, total, total_pages, current_page) = repo
            .get_all::<Role>(filter, &searchable, limit, skip, extra_match)
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    pub async fn update_role(&self, id: &IdType, update: &RolePartial) -> Result<Role, AppError> {
        let existing_role = self.find_one(Some(id), None).await?;

        if existing_role.role_type == RoleType::System {
            return Err(AppError {
                message: "Cannot modify system roles".into(),
            });
        }

        if let Some(ref name) = update.name {
            if name.trim().is_empty() {
                return Err(AppError {
                    message: "Role name cannot be empty".into(),
                });
            }

            if existing_role.name != *name {
                let mut filter = doc! { "name": name };
                if let Some(school_id) = existing_role.school_id {
                    filter.insert("school_id", school_id);
                }

                if self.find_one(None, Some(filter)).await.is_ok() {
                    return Err(AppError {
                        message: format!("Role with name '{}' already exists", name),
                    });
                }
            }
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.update_one_and_fetch::<Role>(id, extract_valid_fields(Role::from_partial(update.clone())?))
            .await
    }

    pub async fn delete_role(&self, id: &IdType) -> Result<Role, AppError> {
        let role = self.find_one(Some(id), None).await?;

        if role.role_type == RoleType::System {
            return Err(AppError {
                message: "Cannot delete system roles".into(),
            });
        }

        let role_oid = IdType::to_object_id(id)?;
        let assignment_count = self
            .assignments_collection
            .count_documents(doc! { "role_id": role_oid })
            .await
            .unwrap_or(0);

        if assignment_count > 0 {
            return Err(AppError {
                message: format!(
                    "Cannot delete role. It is assigned to {} user(s)",
                    assignment_count
                ),
            });
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.delete_one(id).await?;

        Ok(role)
    }

    pub async fn assign_role_to_user(
        &self,
        user_id: &IdType,
        role_id: &IdType,
        school_id: &IdType,
    ) -> Result<UserRoleAssignment, AppError> {
        let role = self.find_one(Some(role_id), None).await?;

        if !role.is_active {
            return Err(AppError {
                message: "Cannot assign inactive role".into(),
            });
        }

        let role_school_id = role.school_id.map(|id| id.to_hex());
        let target_school_id = IdType::to_object_id(school_id)?.to_hex();

        if let Some(rs_id) = role_school_id {
            if rs_id != target_school_id {
                return Err(AppError {
                    message: "Cannot assign role from different school".into(),
                });
            }
        }

        let user_oid = IdType::to_object_id(user_id)?;
        let role_oid = IdType::to_object_id(role_id)?;
        let school_oid = IdType::to_object_id(school_id)?;

        let existing = self
            .assignments_collection
            .find_one(doc! {
                "user_id": user_oid,
                "role_id": role_oid,
                "school_id": school_oid
            })
            .await
            .ok()
            .flatten();

        if existing.is_some() {
            return Err(AppError {
                message: "User already has this role assigned".into(),
            });
        }

        let assignment = UserRoleAssignment {
            id: None,
            user_id: user_oid,
            role_id: role_oid,
            school_id: school_oid,
            assigned_at: chrono::Utc::now(),
        };

        let repo = BaseRepository::new(
            self.assignments_collection
                .clone()
                .clone_with_type::<Document>(),
        );

        let doc = mongodb::bson::to_document(&assignment).map_err(|e| AppError {
            message: format!("Serialization error: {}", e),
        })?;

        repo.create::<UserRoleAssignment>(extract_valid_fields(doc), None)
            .await
    }

    pub async fn user_has_permission(
        &self,
        user_id: &IdType,
        school_id: &IdType,
        permission: &str,
    ) -> Result<bool, AppError> {
        let user_oid = IdType::to_object_id(user_id)?;
        let school_oid = IdType::to_object_id(school_id)?;

        let assignments: Vec<UserRoleAssignment> = self
            .assignments_collection
            .find(doc! {
                "user_id": user_oid,
                "school_id": school_oid
            })
            .await
            .map_err(|e| AppError {
                message: format!("Database error: {}", e),
            })?
            .try_collect()
            .await
            .map_err(|e| AppError {
                message: format!("Collection error: {}", e),
            })?;

        for assignment in assignments {
            let role = self
                .find_one(Some(&IdType::ObjectId(assignment.role_id)), None)
                .await?;

            if role.is_active && role.permissions.contains(&permission.to_string()) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<RoleWithRelations>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(f) = filter {
            let search = build_search_filter(
                Some(f),
                &["name", "description", "_id", "school_id", "role_type"],
            );

            match_stage.extend(search);
        }

        let pipeline = role_pipeline(match_stage);

        repo.aggregate_with_paginate::<RoleWithRelations>(pipeline, limit, skip)
            .await
    }

    pub async fn find_one_with_relations(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<RoleWithRelations, AppError> {
        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(id) = id {
            match_stage.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.aggregate_one::<RoleWithRelations>(role_pipeline(match_stage), None)
            .await?
            .ok_or(AppError {
                message: "Role not found".into(),
            })
    }

    pub async fn count_roles(
        &self,
        filter: Option<String>,
        extra_match: Option<Document>,
    ) -> Result<CountDoc, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        let searchable = ["name", "description", "school_id", "role_type"];

        repo.count(filter, &searchable, extra_match).await
    }

    pub fn get_default_permissions() -> Vec<Permission> {
        vec![
            Permission {
                name: "assignment.create".to_string(),
                description: Some("Create assignments".to_string()),
                scope: PermissionScope::School,
            },
            Permission {
                name: "assignment.read.own".to_string(),
                description: Some("Read own assignments".to_string()),
                scope: PermissionScope::Own,
            },
            Permission {
                name: "assignment.read.class".to_string(),
                description: Some("Read class assignments".to_string()),
                scope: PermissionScope::Class,
            },
            Permission {
                name: "assignment.read.school".to_string(),
                description: Some("Read all school assignments".to_string()),
                scope: PermissionScope::School,
            },
            Permission {
                name: "assignment.update".to_string(),
                description: Some("Update assignments".to_string()),
                scope: PermissionScope::School,
            },
            Permission {
                name: "assignment.delete".to_string(),
                description: Some("Delete assignments".to_string()),
                scope: PermissionScope::School,
            },
            Permission {
                name: "submission.grade".to_string(),
                description: Some("Grade submissions".to_string()),
                scope: PermissionScope::Class,
            },
            Permission {
                name: "submission.read.own".to_string(),
                description: Some("Read own submissions".to_string()),
                scope: PermissionScope::Own,
            },
            Permission {
                name: "submission.read.class".to_string(),
                description: Some("Read class submissions".to_string()),
                scope: PermissionScope::Class,
            },
            Permission {
                name: "submission.read.school".to_string(),
                description: Some("Read all school submissions".to_string()),
                scope: PermissionScope::School,
            },
            Permission {
                name: "parent.read.child.assignment".to_string(),
                description: Some("Read child's assignments".to_string()),
                scope: PermissionScope::Own,
            },
            Permission {
                name: "parent.read.child.submission".to_string(),
                description: Some("Read child's submissions".to_string()),
                scope: PermissionScope::Own,
            },
            Permission {
                name: "role.assign".to_string(),
                description: Some("Assign roles to users".to_string()),
                scope: PermissionScope::School,
            },
            Permission {
                name: "role.create".to_string(),
                description: Some("Create new roles".to_string()),
                scope: PermissionScope::School,
            },
            Permission {
                name: "role.update".to_string(),
                description: Some("Update roles".to_string()),
                scope: PermissionScope::School,
            },
            Permission {
                name: "role.delete".to_string(),
                description: Some("Delete roles".to_string()),
                scope: PermissionScope::School,
            },
            Permission {
                name: "feature.toggle".to_string(),
                description: Some("Toggle features".to_string()),
                scope: PermissionScope::School,
            },
        ]
    }
}
