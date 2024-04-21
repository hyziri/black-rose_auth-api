# Contributing

## Getting Started

You will need

- [Rust](https://www.rust-lang.org/learn/get-started)
- [Docker](https://docs.docker.com/engine/install/)
- [sea-orm-cli](https://www.sea-ql.org/SeaORM/docs/generate-entity/sea-orm-cli/)

Starting the application

1. Copy `.env.example` to `.env` and set variables, get ESI variables from [https://developers.eveonline.com/](https://developers.eveonline.com/) and creating an application.
2. Run `sudo docker-compose up -d` to initialize postgres & valkey
3. Start the application with `cargo run`
4. Follow the admin link provided in the terminal to create an admin account

### sea-orm-cli

- Run migrations with `sea-orm-cli migrate up` or `sea-orm-cli migrate down`
- Generate entities with `sea-orm-cli generate entity -o ./entity/src/entities/ --with-serde both --date-time-crate chrono`

## Submitting new features

Check [github issues](https://github.com/blackrose-eve/black-rose_auth-api/issues) for contribution opportunities.

To implement a new feature

1. Fork this repository ([HOWTO](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/working-with-forks/fork-a-repo))
2. Create a new branch with `git checkout -b branch_name`
3. Commit changes to the branch
4. Push to origin main `git push origin branch_name`
5. Create a pull request ([HOWTO](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request))

For any minor fixes open a pull request, include the issue # if necessary.
