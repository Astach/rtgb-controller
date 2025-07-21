use testcontainers::{ImageExt, bollard::models::MountTypeEnum, core::Mount, runners::AsyncRunner};
use testcontainers_modules::{
    nats::Nats,
    postgres::{self, Postgres},
};

struct PostgresHelper;
impl PostgresHelper {
    async fn start_postgres() {
        let container = postgres::Postgres::default()
            .with_db_name("rtgb_scheduler")
            .with_user("service_rtgb_scheduler")
            .with_password("localpassword")
            .with_mount(Mount::bind_mount(
                "../../docker/postgres/postgresql.conf",
                "/etc/postgresql/postgresql.conf",
            ))
            .with_mount(Mount::bind_mount(
                "../../docker/postgres/pg_hba.conf",
                "/etc/postgresql/pg_hba.conf",
            ))
            .with_mount(Mount::bind_mount(
                "../../docker/postgres/ca.crt",
                "/etc/postgresql/ca.crt",
            ))
            .with_mount(Mount::bind_mount(
                "../../docker/postgres/server.crt",
                "/etc/postgresql/server.crt",
            ))
            .with_mount(Mount::bind_mount(
                "../../docker/postgres/server.key",
                "/etc/postgresql/server.key",
            ))
            .start()
            .await;

        let pg_url = format!(
            "postgres://user:password@localhost:{}/db_name?sslmode=verify-full&sslrootcert=./docker/postgres/ca.crt",
            //pg_port
        );

        println!("Postgres URL: {}", pg_url);
    }
}
