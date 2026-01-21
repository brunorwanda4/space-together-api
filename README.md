# space-together-api

space-together-api implements all requests for space-together applications.

## Features

- RESTful API built with [Actix Web](https://actix.rs/)
- MongoDB integration (sync)
- JWT authentication
- User, student, class, attendance, chat, finance, and notification APIs
- Logging with slog and env_logger
- Password hashing with bcrypt and argon2
- Environment configuration via `.env`

## Getting Started

### Prerequisites

- Rust (edition 2021)
- MongoDB instance
- [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

### Setup

1. Clone the repository:
    ```sh
    git clone https://github.com/space-together/space-together-api.git
    cd space-together-api
    ```

2. Copy the example environment file and edit as needed:
    ```sh
    cp .exmple.env .env
    ```

3. Install dependencies:
    ```sh
    cargo fetch
    ```

4. Build and run the API:
    ```sh
    cargo run --bin space-together-api
    ```

## Project Structure

- `src/api/` - API endpoints (auth, students, classes, etc.)
- `src/config/` - Configuration (DB, logger, env)
- `src/domain/` - Domain models
- `src/repositories/` - Data access logic
- `src/services/` - Business logic
- `src/utils/` - Utilities (hashing, JWT, etc.)

## Configuration

Environment variables are loaded from `.env`. See `.exmple.env` for required variables.

## License

Copyright (c) 2026 Space-Together Organization. All Rights Reserved.

This software is the proprietary and confidential property of Space-Together Organization. 
No part of this software may be copied, modified, distributed, or disclosed without prior written permission from the copyright owner.


---

**Authors:** Rwanda Bruno Happyheart <brunorwanda4@gmail.com>
