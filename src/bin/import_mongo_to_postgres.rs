use anyhow::{Context, Result};
use dotenvy::dotenv;
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Bson, Document},
    Client,
};
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::env;

#[derive(Default)]
struct ImportSummary {
    users: u64,
    schools: u64,
    students: u64,
    enrollments: u64,
    review_items: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let mongo_uri = env::var("MONGO_URI").context("MONGO_URI not set")?;
    let main_db_name = env::var("MAIN_DB_NAME").unwrap_or_else(|_| "space_together".to_string());
    let postgres_url = env::var("POSTGRES_URL").context("POSTGRES_URL not set")?;

    let mongo = Client::with_uri_str(&mongo_uri).await?;
    let pg = PgPoolOptions::new()
        .max_connections(5)
        .connect(&postgres_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pg).await?;

    let run_id = ObjectId::new().to_hex();
    sqlx::query("INSERT INTO mongo_import_runs (id, source_main_db_name) VALUES ($1, $2)")
        .bind(&run_id)
        .bind(&main_db_name)
        .execute(&pg)
        .await?;

    let result = run_import(&mongo, &pg, &main_db_name, &run_id).await;
    match result {
        Ok(summary) => {
            let summary_json = serde_json::json!({
                "users": summary.users,
                "schools": summary.schools,
                "students": summary.students,
                "enrollments": summary.enrollments,
                "review_items": summary.review_items
            });

            sqlx::query(
                "UPDATE mongo_import_runs SET status = 'completed', finished_at = now(), summary = $2 WHERE id = $1",
            )
            .bind(&run_id)
            .bind(summary_json)
            .execute(&pg)
            .await?;

            println!(
                "Mongo import completed: users={}, schools={}, students={}, enrollments={}, review_items={}",
                summary.users,
                summary.schools,
                summary.students,
                summary.enrollments,
                summary.review_items
            );
            Ok(())
        }
        Err(err) => {
            sqlx::query(
                "UPDATE mongo_import_runs SET status = 'failed', finished_at = now(), error_message = $2 WHERE id = $1",
            )
            .bind(&run_id)
            .bind(err.to_string())
            .execute(&pg)
            .await?;
            Err(err)
        }
    }
}

async fn run_import(
    mongo: &Client,
    pg: &PgPool,
    main_db_name: &str,
    run_id: &str,
) -> Result<ImportSummary> {
    let mut summary = ImportSummary::default();
    let main_db = mongo.database(main_db_name);

    summary.users = import_users(&main_db, pg, run_id, main_db_name).await?;
    let schools = import_schools(&main_db, pg, run_id, main_db_name).await?;
    summary.schools = schools.len() as u64;

    for school in schools {
        let (students, enrollments, review_items) =
            import_students_for_school(mongo, pg, run_id, &school).await?;
        summary.students += students;
        summary.enrollments += enrollments;
        summary.review_items += review_items;
    }

    Ok(summary)
}

async fn import_users(
    db: &mongodb::Database,
    pg: &PgPool,
    run_id: &str,
    main_db_name: &str,
) -> Result<u64> {
    let collection = db.collection::<Document>("users");
    let mut cursor = collection.find(doc! {}).await?;
    let mut count = 0;

    while let Some(document) = cursor.try_next().await? {
        let Some(id) = document_id(&document) else {
            continue;
        };

        let name = document_string(&document, "name").unwrap_or_else(|| "Unknown user".to_string());
        let email = document_string(&document, "email")
            .unwrap_or_else(|| format!("missing-{id}@space-together.local"));
        let username = document_string(&document, "username");
        let role = bson_to_string(document.get("role"));
        let phone = document_string(&document, "phone");
        let gender = bson_to_string(document.get("gender"));
        let raw_document = document_json(&document)?;

        sqlx::query(
            r#"
            INSERT INTO users (id, name, email, username, password_hash, role, image, image_id, phone, gender, disabled, raw_document)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, COALESCE($11, false), $12)
            ON CONFLICT (id) DO UPDATE SET
              name = EXCLUDED.name,
              email = EXCLUDED.email,
              username = EXCLUDED.username,
              password_hash = EXCLUDED.password_hash,
              role = EXCLUDED.role,
              image = EXCLUDED.image,
              image_id = EXCLUDED.image_id,
              phone = EXCLUDED.phone,
              gender = EXCLUDED.gender,
              disabled = EXCLUDED.disabled,
              raw_document = EXCLUDED.raw_document
            "#,
        )
        .bind(&id)
        .bind(name)
        .bind(email)
        .bind(username)
        .bind(document_string(&document, "password_hash"))
        .bind(role)
        .bind(document_string(&document, "image"))
        .bind(document_string(&document, "image_id"))
        .bind(phone)
        .bind(gender)
        .bind(document_bool(&document, "disable"))
        .bind(raw_document)
        .execute(pg)
        .await?;

        insert_id_map(pg, run_id, main_db_name, "users", &id, "users", &id, None).await?;
        count += 1;
    }

    Ok(count)
}

#[derive(Clone)]
struct ImportedSchool {
    id: String,
    database_name: String,
}

async fn import_schools(
    db: &mongodb::Database,
    pg: &PgPool,
    run_id: &str,
    main_db_name: &str,
) -> Result<Vec<ImportedSchool>> {
    let collection = db.collection::<Document>("schools");
    let mut cursor = collection.find(doc! {}).await?;
    let mut schools = Vec::new();

    while let Some(document) = cursor.try_next().await? {
        let Some(id) = document_id(&document) else {
            continue;
        };

        let username =
            document_string(&document, "username").unwrap_or_else(|| format!("school-{id}"));
        let name = document_string(&document, "name").unwrap_or_else(|| username.clone());
        let database_name =
            document_string(&document, "database_name").unwrap_or_else(|| format!("school_{id}"));
        let raw_document = document_json(&document)?;

        sqlx::query(
            r#"
            INSERT INTO schools (id, username, name, code, logo, logo_id, description, school_type, affiliation, database_name, is_active, raw_document)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, COALESCE($11, true), $12)
            ON CONFLICT (id) DO UPDATE SET
              username = EXCLUDED.username,
              name = EXCLUDED.name,
              code = EXCLUDED.code,
              logo = EXCLUDED.logo,
              logo_id = EXCLUDED.logo_id,
              description = EXCLUDED.description,
              school_type = EXCLUDED.school_type,
              affiliation = EXCLUDED.affiliation,
              database_name = EXCLUDED.database_name,
              is_active = EXCLUDED.is_active,
              raw_document = EXCLUDED.raw_document
            "#,
        )
        .bind(&id)
        .bind(username)
        .bind(name)
        .bind(document_string(&document, "code"))
        .bind(document_string(&document, "logo"))
        .bind(document_string(&document, "logo_id"))
        .bind(document_string(&document, "description"))
        .bind(bson_to_string(document.get("school_type")))
        .bind(bson_to_string(document.get("affiliation")))
        .bind(&database_name)
        .bind(document_bool(&document, "is_active"))
        .bind(raw_document)
        .execute(pg)
        .await?;

        insert_id_map(
            pg,
            run_id,
            main_db_name,
            "schools",
            &id,
            "schools",
            &id,
            Some(&id),
        )
        .await?;
        schools.push(ImportedSchool { id, database_name });
    }

    Ok(schools)
}

async fn import_students_for_school(
    mongo: &Client,
    pg: &PgPool,
    run_id: &str,
    school: &ImportedSchool,
) -> Result<(u64, u64, u64)> {
    let db = mongo.database(&school.database_name);
    let collection = db.collection::<Document>("students");
    let mut cursor = collection.find(doc! {}).await?;
    let mut profiles = 0;
    let mut enrollments = 0;
    let mut review_items = 0;

    while let Some(document) = cursor.try_next().await? {
        let Some(source_student_id) = document_id(&document) else {
            continue;
        };

        let user_id =
            optional_existing_user_id(pg, document_object_id(&document, "user_id")).await?;
        let name =
            document_string(&document, "name").unwrap_or_else(|| "Unknown student".to_string());
        let email = document_string(&document, "email");
        let phone = document_string(&document, "phone");
        let date_of_birth = optional_document_field_json(&document, "date_of_birth")?;
        let raw_document = document_json(&document)?;

        let (resolution, new_review_items) = resolve_student_profile_id(
            pg,
            run_id,
            &school.database_name,
            &source_student_id,
            &school.id,
            user_id.as_deref(),
            &name,
            email.as_deref(),
            phone.as_deref(),
            date_of_birth.as_ref(),
        )
        .await?;
        review_items += new_review_items;

        let student_profile_id = match resolution {
            StudentResolution::Existing(id) => id,
            StudentResolution::Created | StudentResolution::ReviewOnly => {
                let profile_id = source_student_id.clone();
                sqlx::query(
                    r#"
                    INSERT INTO student_profiles (id, user_id, name, email, phone, gender, image, image_id, date_of_birth, raw_document)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                    ON CONFLICT (id) DO UPDATE SET
                      user_id = COALESCE(student_profiles.user_id, EXCLUDED.user_id),
                      name = EXCLUDED.name,
                      email = COALESCE(EXCLUDED.email, student_profiles.email),
                      phone = COALESCE(EXCLUDED.phone, student_profiles.phone),
                      gender = COALESCE(EXCLUDED.gender, student_profiles.gender),
                      image = COALESCE(EXCLUDED.image, student_profiles.image),
                      image_id = COALESCE(EXCLUDED.image_id, student_profiles.image_id),
                      date_of_birth = COALESCE(EXCLUDED.date_of_birth, student_profiles.date_of_birth),
                      raw_document = EXCLUDED.raw_document
                    "#,
                )
                .bind(&profile_id)
                .bind(user_id.as_deref())
                .bind(&name)
                .bind(email.as_deref())
                .bind(phone.as_deref())
                .bind(bson_to_string(document.get("gender")))
                .bind(document_string(&document, "image"))
                .bind(document_string(&document, "image_id"))
                .bind(date_of_birth.clone())
                .bind(raw_document.clone())
                .execute(pg)
                .await?;
                profiles += 1;
                profile_id
            }
        };

        sqlx::query(
            r#"
            INSERT INTO student_school_enrollments (
              id, student_id, school_id, class_id, subclass_id, source_student_id,
              registration_number, admission_year, status, is_active, raw_document
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, COALESCE($9, 'active'), COALESCE($10, true), $11)
            ON CONFLICT (school_id, source_student_id) DO UPDATE SET
              student_id = EXCLUDED.student_id,
              class_id = EXCLUDED.class_id,
              subclass_id = EXCLUDED.subclass_id,
              registration_number = EXCLUDED.registration_number,
              admission_year = EXCLUDED.admission_year,
              status = EXCLUDED.status,
              is_active = EXCLUDED.is_active,
              raw_document = EXCLUDED.raw_document
            "#,
        )
        .bind(&source_student_id)
        .bind(&student_profile_id)
        .bind(&school.id)
        .bind(document_object_id(&document, "class_id"))
        .bind(document_object_id(&document, "subclass_id"))
        .bind(&source_student_id)
        .bind(document_string(&document, "registration_number"))
        .bind(document_i32(&document, "admission_year"))
        .bind(bson_to_string(document.get("status")))
        .bind(document_bool(&document, "is_active"))
        .bind(raw_document)
        .execute(pg)
        .await?;

        insert_id_map(
            pg,
            run_id,
            &school.database_name,
            "students",
            &source_student_id,
            "student_school_enrollments",
            &source_student_id,
            Some(&school.id),
        )
        .await?;
        enrollments += 1;
    }

    Ok((profiles, enrollments, review_items))
}

enum StudentResolution {
    Existing(String),
    Created,
    ReviewOnly,
}

async fn resolve_student_profile_id(
    pg: &PgPool,
    run_id: &str,
    source_database_name: &str,
    source_student_id: &str,
    school_id: &str,
    user_id: Option<&str>,
    name: &str,
    email: Option<&str>,
    phone: Option<&str>,
    date_of_birth: Option<&Value>,
) -> Result<(StudentResolution, u64)> {
    if let Some(user_id) = user_id {
        if let Some(row) = sqlx::query("SELECT id FROM student_profiles WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pg)
            .await?
        {
            let id: String = row.try_get("id")?;
            return Ok((StudentResolution::Existing(id), 0));
        }
    }

    if let (Some(email), Some(date_of_birth)) = (email, date_of_birth) {
        let rows = sqlx::query(
            "SELECT id FROM student_profiles WHERE lower(email) = lower($1) AND lower(name) = lower($2) AND date_of_birth = $3::jsonb",
        )
        .bind(email)
        .bind(name)
        .bind(date_of_birth)
        .fetch_all(pg)
        .await?;

        if rows.len() == 1 {
            let id: String = rows[0].try_get("id")?;
            return Ok((StudentResolution::Existing(id), 0));
        }

        if rows.len() > 1 {
            insert_review_item(
                pg,
                run_id,
                source_database_name,
                source_student_id,
                None,
                school_id,
                "multiple_email_name_date_of_birth_matches",
                serde_json::json!({ "email": email, "name": name, "date_of_birth": date_of_birth }),
            )
            .await?;
            return Ok((StudentResolution::ReviewOnly, 1));
        }
    }

    if let (Some(phone), Some(date_of_birth)) = (phone, date_of_birth) {
        let rows = sqlx::query(
            "SELECT id FROM student_profiles WHERE phone = $1 AND lower(name) = lower($2) AND date_of_birth = $3::jsonb",
        )
        .bind(phone)
        .bind(name)
        .bind(date_of_birth)
        .fetch_all(pg)
        .await?;

        if rows.len() == 1 {
            let id: String = rows[0].try_get("id")?;
            return Ok((StudentResolution::Existing(id), 0));
        }

        if rows.len() > 1 {
            insert_review_item(
                pg,
                run_id,
                source_database_name,
                source_student_id,
                None,
                school_id,
                "multiple_phone_name_date_of_birth_matches",
                serde_json::json!({ "phone": phone, "name": name, "date_of_birth": date_of_birth }),
            )
            .await?;
            return Ok((StudentResolution::ReviewOnly, 1));
        }
    }

    if email.is_some() || phone.is_some() {
        insert_review_item(
            pg,
            run_id,
            source_database_name,
            source_student_id,
            None,
            school_id,
            "identity_match_missing_safe_date_of_birth",
            serde_json::json!({ "email": email, "phone": phone, "name": name }),
        )
        .await?;
        return Ok((StudentResolution::Created, 1));
    }

    Ok((StudentResolution::Created, 0))
}

async fn optional_existing_user_id(pg: &PgPool, user_id: Option<String>) -> Result<Option<String>> {
    let Some(user_id) = user_id else {
        return Ok(None);
    };

    let exists = sqlx::query("SELECT 1 FROM users WHERE id = $1")
        .bind(&user_id)
        .fetch_optional(pg)
        .await?
        .is_some();

    Ok(exists.then_some(user_id))
}

async fn insert_id_map(
    pg: &PgPool,
    import_run_id: &str,
    source_database_name: &str,
    source_collection_name: &str,
    source_id: &str,
    target_table_name: &str,
    target_id: &str,
    school_id: Option<&str>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO mongo_id_map (
          id, import_run_id, source_database_name, source_collection_name,
          source_id, target_table_name, target_id, school_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (import_run_id, source_database_name, source_collection_name, source_id)
        DO UPDATE SET target_table_name = EXCLUDED.target_table_name,
                      target_id = EXCLUDED.target_id,
                      school_id = EXCLUDED.school_id
        "#,
    )
    .bind(ObjectId::new().to_hex())
    .bind(import_run_id)
    .bind(source_database_name)
    .bind(source_collection_name)
    .bind(source_id)
    .bind(target_table_name)
    .bind(target_id)
    .bind(school_id)
    .execute(pg)
    .await?;

    Ok(())
}

async fn insert_review_item(
    pg: &PgPool,
    import_run_id: &str,
    source_database_name: &str,
    source_student_id: &str,
    candidate_student_id: Option<&str>,
    school_id: &str,
    reason: &str,
    evidence: Value,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO student_identity_review_items (
          id, import_run_id, source_database_name, source_student_id,
          candidate_student_id, school_id, reason, evidence
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(ObjectId::new().to_hex())
    .bind(import_run_id)
    .bind(source_database_name)
    .bind(source_student_id)
    .bind(candidate_student_id)
    .bind(school_id)
    .bind(reason)
    .bind(evidence)
    .execute(pg)
    .await?;

    Ok(())
}

fn document_id(document: &Document) -> Option<String> {
    document.get("_id").and_then(bson_object_id)
}

fn document_object_id(document: &Document, key: &str) -> Option<String> {
    document.get(key).and_then(bson_object_id)
}

fn document_string(document: &Document, key: &str) -> Option<String> {
    match document.get(key) {
        Some(Bson::String(value)) if !value.trim().is_empty() => Some(value.clone()),
        Some(Bson::ObjectId(value)) => Some(value.to_hex()),
        _ => None,
    }
}

fn document_bool(document: &Document, key: &str) -> Option<bool> {
    match document.get(key) {
        Some(Bson::Boolean(value)) => Some(*value),
        _ => None,
    }
}

fn document_i32(document: &Document, key: &str) -> Option<i32> {
    match document.get(key) {
        Some(Bson::Int32(value)) => Some(*value),
        Some(Bson::Int64(value)) => i32::try_from(*value).ok(),
        _ => None,
    }
}

fn optional_document_field_json(document: &Document, key: &str) -> Result<Option<Value>> {
    match document.get(key) {
        Some(value) => Ok(Some(serde_json::to_value(value)?)),
        None => Ok(None),
    }
}

fn document_json(document: &Document) -> Result<Value> {
    Ok(serde_json::to_value(document)?)
}

fn bson_object_id(value: &Bson) -> Option<String> {
    match value {
        Bson::ObjectId(id) => Some(id.to_hex()),
        Bson::String(value) if value.len() == 24 => Some(value.clone()),
        _ => None,
    }
}

fn bson_to_string(value: Option<&Bson>) -> Option<String> {
    match value {
        Some(Bson::String(value)) => Some(value.clone()),
        Some(Bson::ObjectId(value)) => Some(value.to_hex()),
        Some(Bson::Boolean(value)) => Some(value.to_string()),
        Some(Bson::Int32(value)) => Some(value.to_string()),
        Some(Bson::Int64(value)) => Some(value.to_string()),
        Some(Bson::Double(value)) => Some(value.to_string()),
        _ => None,
    }
}
