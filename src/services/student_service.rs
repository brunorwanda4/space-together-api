use crate::{domain::student::Student, repositories::student_repo::StudentRepo};
use mongodb::Database;

pub struct StudentService {
    repo: StudentRepo,
}

impl StudentService {
    pub fn new(db: Database) -> Self {
        Self {
            repo: StudentRepo::new(db),
        }
    }

    pub async fn create_student(&self, student: Student) -> mongodb::error::Result<()> {
        self.repo.insert(&student).await
    }

    pub async fn list_students(&self) -> mongodb::error::Result<Vec<Student>> {
        self.repo.find_all().await
    }
}
