use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ========== ENROLLMENT TRENDS ==========
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnrollmentTrend {
    pub month: String, // Format: "YYYY-MM"
    pub total: i64,
}

// ========== ATTENDANCE RATE ==========
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttendanceRate {
    pub attendance_rate: f64,
    pub total_records: i64,
    pub present_count: i64,
}

// ========== PASS/FAIL DISTRIBUTION ==========
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PassFailDistribution {
    pub pass: i64,
    pub fail: i64,
    pub total: i64,
    pub pass_rate: f64,
}

// ========== FEE COLLECTION SUMMARY ==========
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeeCollectionSummary {
    pub total_expected: f64,
    pub total_collected: f64,
    pub total_outstanding: f64,
    pub collection_rate: f64,
}

// ========== TEACHER WORKLOAD ==========
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeacherWorkload {
    pub teacher_id: String,
    pub teacher_name: String,
    pub classes: i64,
    pub subjects: i64,
    pub total_students: i64,
}

// ========== QUERY PARAMETERS ==========
#[derive(Debug, Deserialize, Clone)]
pub struct EnrollmentTrendsQuery {
    pub year: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AttendanceRateQuery {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}
