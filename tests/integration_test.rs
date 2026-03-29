use ai_infra::schema::objects_s::dsl::objects_s;
use ai_infra::schema::objects_s::*;
use ai_infra::divider;
use ai_micro::{calculate_c, calculate_mp, find_all_with_t, find_nearest, helpers};
use chrono::{Duration as ChronoDuration, NaiveDate, NaiveTime};
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::sql_query;
use diesel::{Connection, RunQueryDsl};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use rand::RngExt;
use rust_xlsxwriter::{Format, Workbook};
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;
use testcontainers::{ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

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

    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("Failed to get port");
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

    let start_date = NaiveDate::from_ymd_opt(2023, 1, 1)
        .unwrap()
        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());

    for i in 0..200000 {
        let current_date = start_date + ChronoDuration::seconds(i as i64);

        let rand_val: u32 = rng.random_range(0..100);
        let type_val = match rand_val {
            0..=44 => "ASK",
            45..=89 => "BID",
            _ => "TRADE",
        };

        let p_val: f32 = rng.random_range(10.0..100.0);
        let mut s_val: f32 = rng.random_range(10000.0..200000.0);

        // Prevent values from falling into the default migration gap
        if s_val >= 99999.0 && s_val <= 100000.0 {
            s_val = 100001.0;
        }

        let row = (i + 1) as u32;
        worksheet
            .write_datetime_with_format(row, 0, current_date, &format)
            .unwrap();
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
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to spawn fill_data binary");

    {
        let stdin = fill_proc.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(b"/tmp/test_data_native.xlsx\n")
            .expect("Failed to write to stdin");
        stdin
            .write_all(b"Sheet1\n")
            .expect("Failed to write to stdin");
        stdin.write_all(b"s\n").expect("Failed to write to stdin");
        stdin.write_all(b"\n").expect("Failed to write to stdin");
    }

    let status = fill_proc.wait().expect("Failed to wait on fill_data");
    assert!(status.success(), "fill_data failed");

    let count: i64 = objects_s
        .count()
        .get_result(&mut connection)
        .expect("Error fetching count");

    assert_eq!(
        count, 200000,
        "There should be 200,000 rows in the partitioned table"
    );

    // Mock stdin for Unix systems
    #[cfg(unix)]
    {
        let temp_file_path = "/tmp/mock_stdin.txt";
        let mut mock_file = std::fs::File::create(temp_file_path).expect("Failed to create mock file");
        mock_file.write_all(b"1\n180000.0\n").expect("Failed to write mock input");

        let mock_file_read = std::fs::File::open(temp_file_path).expect("Failed to open mock file");
        unsafe {
            libc::dup2(std::os::fd::AsRawFd::as_raw_fd(&mock_file_read), libc::STDIN_FILENO);
        }
    }

    let _micro_var = helpers::prompt_microstructure_variable(&mut connection)
        .unwrap_or('s');

    let divide_s = helpers::prompt_cutoff_value().unwrap_or(180000.0);

    // Detach the default 100000 partitions to keep the data safe before dropping target partitions
    sql_query("ALTER TABLE objects_s DETACH PARTITION objects_s_100000_below;")
        .execute(&mut connection)
        .unwrap();
    sql_query("ALTER TABLE objects_s DETACH PARTITION objects_s_100000_above;")
        .execute(&mut connection)
        .unwrap();

    let drop_below = format!("DROP TABLE IF EXISTS objects_s_below_{} CASCADE;", divide_s as i64);
    let drop_above = format!("DROP TABLE IF EXISTS objects_s_above_{} CASCADE;", divide_s as i64);

    sql_query(drop_below).execute(&mut connection).unwrap();
    sql_query(drop_above).execute(&mut connection).unwrap();

    divider(&mut connection, divide_s);

    // Insert the data back into the dynamically created partitions
    sql_query("INSERT INTO objects_s SELECT * FROM objects_s_100000_below;")
        .execute(&mut connection)
        .unwrap();
    sql_query("INSERT INTO objects_s SELECT * FROM objects_s_100000_above;")
        .execute(&mut connection)
        .unwrap();

    // Now it's safe to drop the old detached tables
    sql_query("DROP TABLE IF EXISTS objects_s_100000_below;")
        .execute(&mut connection)
        .unwrap();
    sql_query("DROP TABLE IF EXISTS objects_s_100000_above;")
        .execute(&mut connection)
        .unwrap();

    // 7. Calculate mid-price and cost
    let target_type_t = "TRADE".to_string();
    let target_type_a = "ASK".to_string();
    let target_type_b = "BID".to_string();

    println!("Fetching trade objects...");
    let trade_objects =
        find_all_with_t(&mut connection, &target_type_t).expect("Error querying database");

    println!("Calculating costs and updating DB...");
    for item in trade_objects {
        let mut ap = 0.0;
        let mut bp = 0.0;
        let pt = item.p; // Price of the TRADE object

        if let Some(p_val) = find_nearest(&mut connection, item.id, &target_type_a) {
            ap = p_val;
        }

        if let Some(p_val) = find_nearest(&mut connection, item.id, &target_type_b) {
            bp = p_val;
        }

        let mp = calculate_mp(ap, bp);
        let ec = calculate_c(pt, mp);

        diesel::update(objects_s.filter(id.eq(item.id)))
            .set(c.eq(ec))
            .execute(&mut connection)
            .expect("Error updating column c for ObjectS");

        println!("Udating DB... cost{} for id {}", item.c, item.id);
    }
    tokio::time::sleep(Duration::from_secs(10)).await;
    println!("Test sequence completed successfully.");
    tokio::time::sleep(Duration::from_secs(10)).await;
}
