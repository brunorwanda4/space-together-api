use actix_web::{get, web, HttpResponse, Responder};

#[get("/")]
async fn welcome() -> impl Responder {
    let html = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>Space Together API</title>
        <script src="https://cdn.tailwindcss.com"></script>
        <style>
          body {
            font-family: 'Inter', sans-serif;
            background: linear-gradient(to right, #0f172a, #1e293b);
            color: white;
          }
          .api-card:hover {
            transform: scale(1.02);
            transition: all 0.2s ease-in-out;
          }
        </style>
    </head>
    <body class="min-h-screen p-6">
        <div class="max-w-5xl mx-auto">
            <header class="text-center mb-10">
                <h1 class="text-4xl font-bold mb-2 text-indigo-400">🚀 Space Together API</h1>
                <p class="text-gray-300 text-lg">Welcome to the Space Together backend — built for schools, teachers, and students.</p>
                <p class="text-sm text-gray-400 mt-1">Version 1.0</p>
            </header>

            <section class="bg-gray-800 rounded-2xl shadow-xl p-6">
                <h2 class="text-2xl font-semibold mb-4 text-indigo-300">📚 Available APIs</h2>
                <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-4">
                    <div class="api-card bg-gray-900 rounded-xl p-4 border border-gray-700">
                        <h3 class="font-semibold text-lg text-indigo-300">Auth</h3>
                        <p class="text-gray-400 text-sm mb-2">User authentication and session management</p>
                        <code class="text-green-400 text-sm">/auth</code>
                    </div>

                    <div class="api-card bg-gray-900 rounded-xl p-4 border border-gray-700">
                        <h3 class="font-semibold text-lg text-indigo-300">Users</h3>
                        <p class="text-gray-400 text-sm mb-2">User profiles and role management</p>
                        <code class="text-green-400 text-sm">/users</code>
                    </div>

                    <div class="api-card bg-gray-900 rounded-xl p-4 border border-gray-700">
                        <h3 class="font-semibold text-lg text-indigo-300">Schools</h3>
                        <p class="text-gray-400 text-sm mb-2">School registration and info</p>
                        <code class="text-green-400 text-sm">/school</code>
                    </div>

                    <div class="api-card bg-gray-900 rounded-xl p-4 border border-gray-700">
                        <h3 class="font-semibold text-lg text-indigo-300">Students</h3>
                        <p class="text-gray-400 text-sm mb-2">Student profiles, records, and status</p>
                        <code class="text-green-400 text-sm">/students</code>
                    </div>

                    <div class="api-card bg-gray-900 rounded-xl p-4 border border-gray-700">
                        <h3 class="font-semibold text-lg text-indigo-300">Teachers</h3>
                        <p class="text-gray-400 text-sm mb-2">Teacher profiles and subject assignments</p>
                        <code class="text-green-400 text-sm">/teachers</code>
                    </div>

                    <div class="api-card bg-gray-900 rounded-xl p-4 border border-gray-700">
                        <h3 class="font-semibold text-lg text-indigo-300">Subjects</h3>
                        <p class="text-gray-400 text-sm mb-2">Subjects, topics, and grading systems</p>
                        <code class="text-green-400 text-sm">/subjects</code>
                    </div>

                    <div class="api-card bg-gray-900 rounded-xl p-4 border border-gray-700">
                        <h3 class="font-semibold text-lg text-indigo-300">Events</h3>
                        <p class="text-gray-400 text-sm mb-2">Manage school and system events</p>
                        <code class="text-green-400 text-sm">/events</code>
                    </div>

                    <div class="api-card bg-gray-900 rounded-xl p-4 border border-gray-700">
                        <h3 class="font-semibold text-lg text-indigo-300">Join School Request</h3>
                        <p class="text-gray-400 text-sm mb-2">Handle users joining schools</p>
                        <code class="text-green-400 text-sm">/join-school-request</code>
                    </div>

                    <div class="api-card bg-gray-900 rounded-xl p-4 border border-gray-700">
                        <h3 class="font-semibold text-lg text-indigo-300">Database Status</h3>
                        <p class="text-gray-400 text-sm mb-2">Check DB connection and API health</p>
                        <code class="text-green-400 text-sm">/database-status</code>
                    </div>
                </div>
            </section>

            <footer class="text-center text-gray-500 mt-10 text-sm">
                <p>© 2025 Space Together — Built with ❤️ using Actix-Web + Rust</p>
            </footer>
        </div>
    </body>
    </html>
    "#;

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(welcome);
}
