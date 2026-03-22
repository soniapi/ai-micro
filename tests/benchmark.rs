use ai_infra::establish_connection;
use ai_micro::backwards;
use ai_infra::models::NewObjectS;
use diesel::prelude::*;
use std::time::Instant;

#[test]
fn benchmark_backwards() {
    let connection = &mut establish_connection();

    // Insert 1000 dummy records
    use ai_infra::schema::objects_s::dsl::*;

    // Clear out
    diesel::delete(objects_s).execute(connection).unwrap();

    let mut new_objects = Vec::new();
    for i in 1..=1000 {
        let t_val = if i % 10 == 0 { "TARGET".to_string() } else { "OTHER".to_string() };
        new_objects.push(NewObjectS {
            d: chrono::Utc::now().naive_utc(),
            t: t_val,
            p: 1.0,
            s: 1.0,
            c: 1.0,
        });
    }

    diesel::insert_into(objects_s)
        .values(&new_objects)
        .execute(connection)
        .unwrap();

    // Get the max id
    let max_id: i32 = objects_s.select(diesel::dsl::max(id)).first::<Option<i32>>(connection).unwrap().unwrap();

    let target_type = "TARGET".to_string();

    let start = Instant::now();
    for _ in 0..100 {
        let _ = backwards(connection, max_id, &target_type).unwrap();
    }
    let duration = start.elapsed();

    println!("Time taken for 100 iterations of backwards: {:?}", duration);
}
