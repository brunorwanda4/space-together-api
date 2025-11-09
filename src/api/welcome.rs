use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize)]
struct ApiInfo {
    name: &'static str,
    description: &'static str,
    path: &'static str,
}

#[derive(Serialize)]
struct WelcomeResponse {
    app: &'static str,
    version: &'static str,
    message: &'static str,
    apis: Vec<ApiInfo>,
}

#[get("/")]
async fn welcome() -> impl Responder {
    let apis = vec![
        ApiInfo {
            name: "Auth",
            description: "User authentication and session management",
            path: "/auth",
        },
        ApiInfo {
            name: "Users",
            description: "User profiles and roles",
            path: "/users",
        },
        ApiInfo {
            name: "Trade",
            description: "Manage trade and business info for schools",
            path: "/trade",
        },
        ApiInfo {
            name: "Main Class",
            description: "Top-level class management for schools",
            path: "/main-class",
        },
        ApiInfo {
            name: "Classes",
            description: "Manage classes, students, and class info",
            path: "/class",
        },
        ApiInfo {
            name: "Students",
            description: "Student records, status, and performance",
            path: "/students",
        },
        ApiInfo {
            name: "Teachers",
            description: "Teacher profiles, subjects, and assignments",
            path: "/teachers",
        },
        ApiInfo {
            name: "Subjects",
            description: "Main subject management and configuration",
            path: "/subjects",
        },
        ApiInfo {
            name: "Subject Topics",
            description: "Topics inside a subject",
            path: "/subjects/topics",
        },
        ApiInfo {
            name: "Learning Outcomes",
            description: "Define subject learning outcomes",
            path: "/subjects/outcomes",
        },
        ApiInfo {
            name: "Learning Materials",
            description: "Resources for subjects (files, links, notes)",
            path: "/subjects/materials",
        },
        ApiInfo {
            name: "Grading Schemes",
            description: "Manage subject grading and marking",
            path: "/subjects/grading-schemes",
        },
        ApiInfo {
            name: "Events",
            description: "School and system events",
            path: "/events",
        },
        ApiInfo {
            name: "School",
            description: "School registration and info management",
            path: "/school",
        },
        ApiInfo {
            name: "School Collections",
            description: "Group school-related collections (classes, staff, students, etc.)",
            path: "/school-collections",
        },
        ApiInfo {
            name: "Join School Requests",
            description: "Handle user join requests to schools",
            path: "/join-school-request",
        },
        ApiInfo {
            name: "Database Status",
            description: "Check database and API connection health",
            path: "/database-status",
        },
        ApiInfo {
            name: "Sector",
            description: "Manage sectors or regions for schools",
            path: "/sector",
        },
    ];

    let response = WelcomeResponse {
        app: "Space Together API",
        version: "v0.1.0",
        message: "Welcome to Space Together — a platform to connect schools, students, and staff.",
        apis,
    };

    HttpResponse::Ok().json(response)
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(welcome);
}
