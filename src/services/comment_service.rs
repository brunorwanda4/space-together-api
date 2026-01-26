use mongodb::{
    bson::{self, doc, Document},
    Collection, Database,
};

use crate::{
    domain::{
        comment::{Comment, CommentPartial, CommentWithRelations},
        common_details::Paginated,
    },
    errors::AppError,
    models::{
        id_model::IdType,
        mongo_model::{CountDoc, IndexDef},
    },
    pipeline::comment_pipeline::comment_pipeline,
    repositories::base_repo::BaseRepository,
    utils::mongo_utils::extract_valid_fields,
};

pub struct CommentService {
    pub collection: Collection<Comment>,
}

impl CommentService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Comment>("comments"),
        }
    }

    pub async fn ensure_indexes(&self) -> Result<(), AppError> {
        let indexes = vec![
            IndexDef::compound(vec![("target_post_id", 1), ("created_at", 1)], false),
            IndexDef::single("parent_comment_id", false),
            IndexDef::single("author.id", false),
            IndexDef::single("created_at", false),
        ];

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());

        repo.ensure_indexes(&indexes).await?;
        Ok(())
    }

    pub async fn create(&self, dto: Comment) -> Result<Comment, AppError> {
        self.ensure_indexes().await?;

        let partial = dto.to_partial();
        let full_doc = bson::to_document(&partial).map_err(|e| AppError {
            message: format!("Failed to serialize comment: {}", e),
        })?;

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.create::<Comment>(full_doc, None).await
    }

    pub async fn find_one(
        &self,
        id: Option<&IdType>,
        extra_match: Option<Document>,
    ) -> Result<Comment, AppError> {
        let mut filter = extra_match.unwrap_or_default();
        if let Some(id) = id {
            filter.insert("_id", IdType::to_object_id(id)?);
        }

        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.find_one::<Comment>(filter, None)
            .await?
            .ok_or(AppError {
                message: "Comment not found".into(),
            })
    }

    pub async fn get_all(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<Comment>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let searchable = ["content", "_id", "target_post_id"];

        let (data, total, total_pages, current_page) = repo
            .get_all::<Comment>(filter, &searchable, limit, skip, extra_match)
            .await?;

        Ok(Paginated {
            data,
            total,
            total_pages,
            current_page,
        })
    }

    pub async fn update(&self, id: &IdType, update: &CommentPartial) -> Result<Comment, AppError> {
        let full_doc = bson::to_document(update).map_err(|e| AppError {
            message: format!("Serialize update failed: {}", e),
        })?;

        let update_doc = extract_valid_fields(full_doc);
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.update_one_and_fetch::<Comment>(id, update_doc).await
    }

    pub async fn delete(&self, id: &IdType) -> Result<Comment, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let comment = self.find_one(Some(id), None).await?;
        repo.delete_one(id).await?;
        Ok(comment)
    }

    pub async fn get_all_with_relations(
        &self,
        filter: Option<String>,
        limit: Option<i64>,
        skip: Option<i64>,
        extra_match: Option<Document>,
    ) -> Result<Paginated<CommentWithRelations>, AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let mut match_stage = extra_match.unwrap_or_default();

        if let Some(f) = filter {
            match_stage.insert(
                "$or",
                vec![doc! { "content": { "$regex": &f, "$options": "i" } }],
            );
        }

        let pipeline = comment_pipeline(match_stage);
        repo.aggregate_with_paginate::<CommentWithRelations>(pipeline, limit, skip)
            .await
    }

    pub async fn count_comments(
        &self,
        filter: Option<String>,
        extra_match: Option<Document>,
    ) -> Result<CountDoc, AppError> {
        let base_repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        let searchable = ["content", "target_post_id", "author.id"];
        let total = base_repo.count(filter, &searchable, extra_match).await?;
        Ok(total)
    }

    pub async fn delete_many(&self, filter: Document) -> Result<(), AppError> {
        let repo = BaseRepository::new(self.collection.clone().clone_with_type::<Document>());
        repo.delete_many(filter).await?;
        Ok(())
    }
}
