# Todos

## Development

This project uses sqlx for database interactions. To set up the development environment, follow these steps:
1. Install SQLx CLI: `cargo install sqlx-cli`
2. Create an empty db file: `touch tasks.db`
3. Create a `.env` file in the project root with the following content:
```
DATABASE_URL="sqlite://./tasks.db"
```
4. Run database migrations: `sqlx migrate run`
