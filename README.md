# Netflix Backend Rust

This project is the backend for a Netflix clone built with Rust, using Actix-web for the web server and MongoDB for the database. It handles user authentication, movie listings, user profiles, and more.

## Features

- User authentication (login and registration)
- CRUD operations for movies and lists
- User profile management
- Secure handling of JWTs for session management

## Prerequisites

Before you begin, ensure you have met the following requirements:
- Rust 1.80 or higher
- Docker and Docker Compose
- MongoDB account and database

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes.

### Environment Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/abdulwaarith0/netflix_backend_rust.git
   cd netflix_backend_rust
   ```

2. Set up your `.env` file based on the `.env.example` provided in the repository. Make sure to replace the placeholders with your actual MongoDB URL and secret key.

### Running with Docker

To run the application using Docker, follow these steps:

1. Build the Docker images:
   ```bash
   make build
   ```

2. Start the application:
   ```bash
   make up
   ```

3. To stop the application:
   ```bash
   make down
   ```

4. For a complete cleanup (removing all containers, networks, and volumes):
   ```bash
   make clean
   ```

### Running Locally without Docker

If you prefer to run the application without Docker:

1. Install MongoDB locally or set up a remote MongoDB instance.
2. Install the required Rust dependencies:
   ```bash
   cargo build
   ```
3. Run the application:
   ```bash
   cargo run
   ```


## API Endpoints
Below are the available RESTful endpoints grouped by resource.

### Authentication

| Method | Endpoint                | Description               | Requires Auth |
|--------|-------------------------|---------------------------|---------------|
| POST   | `/api/auth/login`       | Logs in a user            | No            |
| POST   | `/api/auth/register`    | Registers a new user      | No            |

### Movies

| Method | Endpoint                | Description                        | Requires Auth |
|--------|-------------------------|------------------------------------|---------------|
| GET    | `/api/movies`           | Retrieves all movies               | Yes           |
| POST   | `/api/movies`           | Adds a new movie                   | Yes           |
| GET    | `/api/movies/{id}`      | Retrieves a movie by ID            | Yes           |
| GET    | `/api/movies/random`    | Retrieves a random movie           | No            |

### Users

| Method | Endpoint          | Description               | Requires Auth |
|--------|-------------------|---------------------------|---------------|
| GET    | `/api/users`      | Fetches all users         | Yes           |
| GET    | `/api/users/{id}` | Fetches a specific user   | Yes           |

### Health Check

| Method | Endpoint          | Description               | Requires Auth |
|--------|-------------------|---------------------------|---------------|
| GET    | `/api/health`     | Checks service health     | No            |
