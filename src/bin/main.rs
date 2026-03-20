use core::f32;

use diesel::BoolExpressionMethods;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use ai_infra::establish_connection;
use ai_infra::schema::objects_s::dsl::objects_s;
use ai_infra::schema::objects_s::*;
use ai_infra::models::ObjectS;
use::ai_micro::*;
use diesel::dsl::avg;
use diesel::dsl::max;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection = &mut establish_connection();

    let target_type_a = "ASK".to_string();
    let target_type_b = "BID".to_string();
    let target_type_t = "TRADE".to_string();
    let mut avg_value_population: f32 = 0.0;
    let mut ap = 0.0;
    let mut bp = 0.0;
    let pt = 0.0;

     
    match find_all_with_t(connection, &target_type_t) {
        Ok(trade_objects) => {
            for item in trade_objects {
                println!("Item id {:?}", item.id);
                let start_object_id = item.id;

                match micro::backwards(connection, start_object_id, &target_type_a) {
                    Ok(items) => {
                        for item in items {
                            println!("Found object: id={:?}, type={:?}, date={:?}", item.id, item.t, item.d);
                            ap = item.p;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error fetching objects: {}", e);
                    }
                }

                match micro::backwards(connection, start_object_id, &target_type_b) {
                    Ok(items) => {
                        for item in items {
                            println!("Found object: id={:?}, type={:?}, date={:?}", item.id, item.t, item.d);                
                            bp = item.p;
                         }
                    }
                    Err(e) => {
                        eprintln!("Error fetching objects: {}", e);
                    }
                }

                let mp = calculate_mp(&ap, &bp);
                let ec = calculate_c(&pt, &mp);       
                println!("c is {:?}", &ec);

                diesel::update(objects_s.filter(id.eq(start_object_id)))
                    .set(c.eq(ec))
                    .returning(ObjectS::as_select())
                    .get_result(connection)
                    .expect("Error updating column ce for ObjectS");

                let result: f32 = objects_s
                    .select(c) 
                    .filter(id.eq(start_object_id))
                    .first(connection) 
                    .expect("Error loading object ec");

                println!("Column c: {:?}", result);
            }
        }
        Err(e) => {
            eprintln!("Error fetching objects: {:?}", e);
        }
    }

    let result: Result<Option<f64>, diesel::result::Error> = objects_s
        .select(avg(c))
        .first(connection);

    match result {
        Ok(Some(average_value)) => { 
            println!("Whole population average: {:?}", average_value);
            avg_value_population = average_value as f32;
        }
        Ok(None) => println!("No data found to calculate the average."),
        Err(e) => eprintln!("Error calculating average: {:?}", e),
    }

    let start_s = 0.0_f32;
    let divide_s = 195000.0_f32;

    // Something else
    let result_1: Result<Option<f64>, diesel::result::Error> = objects_s
        .filter(s.ge(start_s).and(s.lt(divide_s)))
        .select(avg(c))
        .first(connection);

    match result_1 {
        Ok(Some(average_value_1)) => println!("Population_1 c average: {:?}", average_value_1),
        Ok(None) => println!("No data found to calculate the c average."),
        Err(e) => eprintln!("Error calculating c average: {:?}", e),
    }

    let result_max: Result<Option<f32>, diesel::result::Error> = objects_s
        .select(max(s))
        .first(connection);

    match result_max {
        Ok(Some(max_value)) => {
            println!("Whole population max s: {:?}", max_value);

            let result_2: Result<Option<f64>, diesel::result::Error> = objects_s
                .filter(s.ge(divide_s).and(s.le(max_value)))
                .select(avg(c))
                .first(connection);

            match result_2 {
                Ok(Some(average_value_2)) => println!("Population_2 c average: {:?}", average_value_2),
                Ok(None) => println!("No data found to calculate the c average."),
                Err(e) => eprintln!("Error calculating c average: {:?}", e),
            }

        }
        Ok(None) => println!("No data found to calculate the max."),
        Err(e) => eprintln!("Error calculating max: {:?}", e),
    }

    let n: f32 = objects_s.count().get_result::<i64>(connection)? as f32;
    println!("Whole Population count n: {:?}", n);

    let m: f32 = objects_s.filter(c.lt(avg_value_population)).count().get_result::<i64>(connection)? as f32;
    println!("Whole population m: {:?}", m);

    let n1: f32 = objects_s.filter(s.lt(195000.00)).count().get_result::<i64>(connection)? as f32;
    println!("Population 1 count n1: {:?}", n1);

    let m1: f32 = objects_s.filter(s.lt(195000.00)).filter(c.lt(avg_value_population)).count().get_result::<i64>(connection)? as f32;
    println!("Population 1 m1: {:?}", m1);

    let n2: f32 = objects_s.filter(s.ge(195000.00)).count().get_result::<i64>(connection)? as f32;
    println!("Population 2 count n2: {:?}", n2);

    let m2: f32 = objects_s.filter(s.ge(195000.00)).filter(c.lt(avg_value_population)).count().get_result::<i64>(connection)? as f32;
    println!("Population 2 m2: {:?}", m2);

    let (_p_temp, p1, p2) = prop::calculate_proportions(&m, &m1, &m2, &n, &n1, &n2);

    let pooled_estimate = prop::calculate_pooled_estimate (&n1, &n2, &p1, &p2);

    prop::calculate_z_statistics(&n1, &n2, &p1, &p2, &pooled_estimate);

  Ok(())

}
