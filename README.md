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
├── db/
├── middleware/
│ └── rate_limiter.rs
├── routes/
│ ├── health.rs
│ ├── rate_test.rs
│ └── user.rs
├── error.rs
├── logger.rs
└── main.rs
```


## Getting Started

1. Clone the repository
2. Install Rust and Cargo
3. Set up PostgreSQL and create a database
4. Update the database configuration in `src/main.rs`:

5. Run the project:

The server will start on `http://127.0.0.1:8080`.

## API Endpoints

- `GET /api/health`: Health check endpoint
- `GET /api/rate-test`: Rate limiting test endpoint
- `POST /api/register`: User registration endpoint

## Configuration

- Database settings: Update in `src/main.rs`
- Logging: Configure in `src/main.rs` and `src/logger.rs`
- Rate limiting: Adjust in `src/middleware/rate_limiter.rs`

## Todo

- [ ] Add authentication middleware
- [ ] Implement more CRUD operations
- [ ] Add unit and integration tests
- [ ] Set up CI/CD pipeline
- [ ] Add database migrations
- [ ] Implement caching mechanism

## License

This project is for personal use, however, if you find it useful, please feel free to use it.