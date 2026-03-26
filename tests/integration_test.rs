use ai_infra::{establish_connection, divider};
use ai_micro::{find_all_with_t, calculate_mp, calculate_c, backwards};
use diesel::{RunQueryDsl, Connection};
use ai_infra::schema::objects_s::dsl::objects_s;
use ai_infra::schema::objects_s::*;
use diesel::QueryDsl;
use ai_infra::models::ObjectS;
use diesel::ExpressionMethods;
use std::time::Duration;
use diesel::sql_query;
use testcontainers::{runners::AsyncRunner, ImageExt};
use testcontainers_modules::postgres::Postgres;
use rust_xlsxwriter::{Workbook, Format};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Duration as ChronoDuration};
use rand::{Rng, RngExt};
use std::process::{Command, Stdio};
use std::io::Write;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../ai-infra/migrations");

#[tokio::test]
async fn test_end_to_end_sequence() {
    let container = Postgres::default()
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_env_var("POSTGRES_DB", "postgres")
        .start()
        .await
        .expect("Failed to start Postgres container");

    let port = container.get_host_port_ipv4(5432).await.expect("Failed to get port");
    let database_url = format!("postgres://postgres:postgres@localhost:{}/postgres", port);

    unsafe {
        std::env::set_var("DATABASE_URL", database_url);
    }

    let mut connection;
    let mut retry = 0;
    loop {
        if let Ok(conn) = diesel::PgConnection::establish(&std::env::var("DATABASE_URL").unwrap()) {
            connection = conn;
            break;
        }
        if retry > 10 {
            panic!("Could not connect to database.");
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
        retry += 1;
    }

    connection.run_pending_migrations(MIGRATIONS).unwrap();

    sql_query("DROP TABLE IF EXISTS objects_s_100000_below CASCADE;").execute(&mut connection).unwrap();
    sql_query("DROP TABLE IF EXISTS objects_s_100000_above CASCADE;").execute(&mut connection).unwrap();

    divider(&mut connection, 180000.0);

    // 6. Generate 200,000 rows natively in Rust
    println!("Generating test data using rust_xlsxwriter...");
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet().set_name("Sheet1").unwrap();

    // Headers
    worksheet.write(0, 0, "d").unwrap();
    worksheet.write(0, 1, "t").unwrap();
    worksheet.write(0, 2, "p").unwrap();
    worksheet.write(0, 3, "s").unwrap();

    let mut rng = rand::rng();
    let format = Format::new().set_num_format("yyyy-mm-dd hh:mm:ss");

    let start_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());

    for i in 0..200000 {
        let current_date = start_date + ChronoDuration::seconds(i as i64);

        let rand_val: u32 = rng.random_range(0..100);
        let type_val = match rand_val {
            0..=44 => "ASK",
            45..=89 => "BID",
            _ => "TRADE",
        };

        let p_val: f32 = rng.random_range(10.0..100.0);
        let s_val: f32 = rng.random_range(10000.0..200000.0);

        let row = (i + 1) as u32;
        worksheet.write_datetime_with_format(row, 0, current_date, &format).unwrap();
        worksheet.write(row, 1, type_val).unwrap();
        worksheet.write(row, 2, p_val).unwrap();
        worksheet.write(row, 3, s_val).unwrap();
    }

    workbook.save("/tmp/test_data_native.xlsx").unwrap();
    println!("Data generated natively");

    println!("Filling partitions with 200,000 rows. This might take a bit...");

    let mut fill_proc = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("fill_data")
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .spawn()
        .expect("Failed to spawn fill_data binary");

    {
        let stdin = fill_proc.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(b"/tmp/test_data_native.xlsx\n").expect("Failed to write to stdin");
        stdin.write_all(b"Sheet1\n").expect("Failed to write to stdin");
        stdin.write_all(b"s\n").expect("Failed to write to stdin");
        stdin.write_all(b"\n").expect("Failed to write to stdin");
    }

    let status = fill_proc.wait().expect("Failed to wait on fill_data");
    assert!(status.success(), "fill_data failed");

    let count: i64 = objects_s
        .count()
        .get_result(&mut connection)
        .expect("Error fetching count");

    assert_eq!(count, 200000, "There should be 200,000 rows in the partitioned table");

    // 7. Calculate mid-price and cost
    let target_type_t = "TRADE".to_string();
    let target_type_a = "ASK".to_string();
    let target_type_b = "BID".to_string();

    println!("Fetching trade objects...");
    let trade_objects = find_all_with_t(&mut connection, &target_type_t).expect("Error querying database");

    println!("Calculating costs and updating DB...");
    for item in trade_objects {
        let mut ap = 0.0;
        let mut bp = 0.0;
        let pt = item.p; // Price of the TRADE object

        if let Ok(items) = backwards(&mut connection, item.id, &target_type_a) {
            if let Some(first_ask) = items.first() {
                ap = first_ask.p;
            }
        }

        if let Ok(items) = backwards(&mut connection, item.id, &target_type_b) {
            if let Some(first_bid) = items.first() {
                bp = first_bid.p;
            }
        }

        let mp = calculate_mp(ap, bp);
        let ec = calculate_c(pt, mp);

        diesel::update(objects_s.filter(id.eq(item.id)))
            .set(c.eq(ec))
            .execute(&mut connection)
            .expect("Error updating column c for ObjectS");
    }

    println!("Test sequence completed successfully.");
}
