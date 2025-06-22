# Scalable Actix Web API Starter

This project serves as a robust starting point for building scalable web APIs using Rust and Actix Web. It's designed for personal use and includes essential features to kickstart development of any project intended to grow and scale.

## Features

- Actix Web framework for high-performance web services
- PostgreSQL database integration with connection pooling
- Rate limiting middleware
- Logging setup with log4rs
- User authentication (registration endpoint)
- Error handling
- Modular project structure

## Project Structure

```
src/
├── auth/
├── config/
├── db/
├── error/
├── logger/
├── middleware/
│ └── rate_limiter.rs
├── routes/
│ ├── health.rs
│ ├── rate_test.rs
│ ├── user.rs
│ └── statistics.rs
├── statistics/
├── main.rs
└── lib.rs
```



## Getting Started

1. Clone the repository
2. Install Rust and Cargo
3. Set up PostgreSQL and create a database
4. Copy `config.example.toml` to `config.toml` and update the configuration:

   ```
   cp config.example.toml config.toml
   ```

   Edit `config.toml` with your specific settings.

5. Run database migrations:

   ```
   sqlx migrate run
   ```

6. Run the project:

   ```
   cargo run
   ```

The server will start on the host and port specified in your `config.toml`.

## API Endpoints

- `GET /api/health`: Health check endpoint
- `GET /api/rate-test`: Rate limiting test endpoint
- `POST /api/register`: User registration endpoint
- `GET /api/user/{user_id}`: Get user information
- `GET /api/statistics`: Get API usage statistics
- `GET /api/system_health`: Get system health information

## Configuration

The application uses a `config.toml` file for configuration. You can adjust the following settings:

- Database connection
- Server host and port
- Rate limiting parameters
- Logging level and file location

Refer to `config.toml` for available options.

## Database Migrations

This project uses SQLx for database migrations. To create a new migration:


## API Endpoints

- `GET /api/health`: Health check endpoint
- `GET /api/rate-test`: Rate limiting test endpoint
- `POST /api/register`: User registration endpoint
- `GET /api/user/{user_id}`: Get user information
- `GET /api/statistics`: Get API usage statistics
- `GET /api/system_health`: Get system health information

To run migrations:

```
sqlx migrate add <migration_name>
sqlx migrate run
```

## Statistics and Monitoring

The API includes endpoints for monitoring its performance and usage:

- `/api/statistics`: Provides information about API usage, response times, and error rates.
- `/api/system_health`: Offers insights into system resources like CPU, memory, and disk usage.

## Error Handling

The project uses a custom `AppError` type for consistent error handling across the application. This ensures that all errors are properly logged and returned to the client in a standardized format.

## Rate Limiting

Rate limiting is implemented using the `governor` crate. The limits are configurable in the `config.toml` file.

## Authentication

User authentication is handled using JWT tokens. The `/api/register` endpoint creates new users and returns a token, which should be included in the Authorization header for protected routes.

## CORS

Cross-Origin Resource Sharing (CORS) is enabled and configured to be permissive by default. Adjust the CORS settings in `main.rs` as needed for your production environment.

## Logging

Logging is set up using `log4rs` and can be configured in the `log4rs.yaml` file. Logs include request details, rate limiting information, and other important events.

## Todo

- [ ] Implement more CRUD operations
- [ ✓ ] Add unit and integration tests
- [ ] Set up CI/CD pipeline
- [ ] Implement caching mechanism
- [ ] Add more advanced authentication features (login, logout, password reset)
- [ ] Implement request validation using a crate like `validator`
- [ ] Add API documentation using Swagger/OpenAPI

## License

This project is for personal use, however, if you find it useful, please feel free to use it.
