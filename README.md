# Black Rose Auth API

## Development

1. Copy `.env.example` to `.env` and set variables
2. Run `sudo docker-compose up -d` to initialize postgres & redis
3. Run `cargo run`
4. Navigate to `localhost:8080/docs` to see API routes schema

Note: Login route don't work on swagger interface, to login you will need to navigate to it via your web browser: `localhost:8080/auth/login`.

### Sea-ORM

- `sea-orm-cli migrate up` to run migrations or `sea-orm-cli migrate down` to remove them
- Generate entities with `sea-orm-cli generate entity -o ./entity/src/entities/ --with-serde both --date-time-crate chrono`
