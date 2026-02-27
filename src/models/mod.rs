use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

use chrono::Utc;

use crate::auth::models::{CreateSchoolRequest, School, User};
use crate::errors::AppError;

#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub enum PaymentStatus {
    Paid,
    Pending,
}

#[derive(Clone, Serialize)]
pub struct Student {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub status: PaymentStatus,
    pub department: String,
    pub payment_reference: Option<String>, // tracks Paystack transaction ref
}

#[derive(Deserialize)]
pub struct CreateStudentRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub department: String,
}

#[derive(Clone)]
pub struct AppStore {
    pub students: Arc<Mutex<HashMap<String, Student>>>,
    pub users: Arc<Mutex<HashMap<String, User>>>,
    pub schools: Arc<Mutex<HashMap<String, School>>>,
}

impl AppStore {
    pub fn new() -> Self {
        Self {
            students: Arc::new(Mutex::new(HashMap::new())),
            users: Arc::new(Mutex::new(HashMap::new())),
            schools: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_student(&self, student: CreateStudentRequest) -> Result<(), AppError> {
        let new_student = Student {
            id: Uuid::new_v4(),
            first_name: student.first_name,
            last_name: student.last_name,
            email: student.email,
            department: student.department,
            status: PaymentStatus::Pending,
            payment_reference: None,
        };

        self.students
            .lock()
            .await
            .insert(new_student.id.to_string(), new_student);

        Ok(())
    }

    pub async fn get_all_students(&self) -> Result<Vec<Student>, AppError> {
        Ok(self.students.lock().await.values().cloned().collect())
    }

    pub async fn delete_student(&self, id: Uuid) -> Result<(), AppError> {
        self.students.lock().await.remove(&id.to_string());
        Ok(())
    }

    pub async fn get_student(&self, id: Uuid) -> Result<Student, AppError> {
        if let Some(student) = self.students.lock().await.get(&id.to_string()) {
            Ok(student.clone())
        } else {
            Err(AppError::NotFound)
        }
    }

    // Save the Paystack reference against the student after initializing payment
    pub async fn set_payment_reference(&self, id: Uuid, reference: String) -> Result<(), AppError> {
        let mut store = self.students.lock().await;
        if let Some(student) = store.get_mut(&id.to_string()) {
            student.payment_reference = Some(reference);
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }

    // Called from the webhook handler to mark student as Paid
    pub async fn mark_student_paid_by_reference(&self, reference: &str) -> Result<(), AppError> {
        let mut store = self.students.lock().await;
        let student = store
            .values_mut()
            .find(|s| s.payment_reference.as_deref() == Some(reference));

        if let Some(student) = student {
            student.status = PaymentStatus::Paid;
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }

    pub async fn create_user(&self, user: User) -> Result<User, AppError> {
        let mut users = self.users.lock().await;
        let email_key = user.email.to_lowercase();
        if users.contains_key(&email_key) {
            return Err(AppError::UnProcessableEntity {
                field: "email".to_string(),
                message: "Email already exists".to_string(),
            });
        }
        users.insert(email_key, user.clone());
        Ok(user)
    }

    pub async fn find_user_by_email(&self, email: &str) -> Option<User> {
        let users = self.users.lock().await;
        users.get(&email.to_lowercase()).cloned()
    }

    pub async fn find_user_by_id(&self, id: Uuid) -> Option<User> {
        let users = self.users.lock().await;
        users.values().find(|u| u.id == id).cloned()
    }

    pub async fn get_all_users(&self) -> Vec<User> {
        let users = self.users.lock().await;
        users.values().cloned().collect()
    }

    pub async fn create_school(
        &self,
        req: CreateSchoolRequest,
    ) -> Result<School, AppError> {
        let mut schools = self.schools.lock().await;
        let slug = slugify(&req.name);
        if slug.is_empty() {
            return Err(AppError::UnProcessableEntity {
                field: "name".to_string(),
                message: "School name is invalid".to_string(),
            });
        }

        if schools.values().any(|school| school.slug == slug) {
            return Err(AppError::UnProcessableEntity {
                field: "name".to_string(),
                message: "School already exists".to_string(),
            });
        }

        let school = School {
            id: Uuid::new_v4(),
            name: req.name,
            slug: slug.clone(),
            is_active: true,
            created_at: Utc::now().timestamp(),
        };
        schools.insert(school.id.to_string(), school.clone());
        Ok(school)
    }

    pub async fn school_exists(&self, id: Uuid) -> bool {
        let schools = self.schools.lock().await;
        schools.contains_key(&id.to_string())
    }

    pub async fn get_all_schools(&self) -> Vec<School> {
        let schools = self.schools.lock().await;
        schools.values().cloned().collect()
    }
}

fn slugify(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
