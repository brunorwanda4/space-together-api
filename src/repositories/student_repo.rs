use crate::domain::student::Student;
use futures::StreamExt;
use mongodb::{bson::doc, Database};

pub struct StudentRepo {
    db: Database,
}

impl StudentRepo {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn insert(&self, student: &Student) -> mongodb::error::Result<()> {
        let collection = self.db.collection::<Student>("students");
        collection.insert_one(student).await?;
        Ok(())
    }

    pub async fn find_all(&self) -> mongodb::error::Result<Vec<Student>> {
        let collection = self.db.collection::<Student>("students");
        let mut cursor = collection.find(doc! {}).await?;
        let mut students = Vec::new();
        while let Some(result) = cursor.next().await {
            students.push(result?);
        }
        Ok(students)
    }
}
