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
└── main.rs
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

5. Run the project:

   ```
   cargo run
   ```

The server will start on the host and port specified in your `config.toml`.

## API Endpoints

- `GET /api/health`: Health check endpoint
- `GET /api/rate-test`: Rate limiting test endpoint
- `POST /api/register`: User registration endpoint

## Configuration

The application uses a `config.toml` file for configuration. You can adjust the following settings:

- Database connection
- Server host and port
- Rate limiting parameters
- Logging level and file location

Refer to `config.toml` for available options.

## Todo

- [ ] Add authentication middleware
- [ ] Implement more CRUD operations
- [ ] Add unit and integration tests
- [ ] Set up CI/CD pipeline
- [ ] Add database migrations
- [ ] Implement caching mechanism

## License

This project is for personal use, however, if you find it useful, please feel free to use it.