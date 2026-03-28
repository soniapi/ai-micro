use core::f32;

use ::ai_micro::*;
use ai_infra::establish_connection;
use ai_infra::models::ObjectS;
use ai_infra::schema::objects_s::dsl::objects_s;
use ai_infra::schema::objects_s::*;
use diesel::BoolExpressionMethods;
use diesel::dsl::avg;
use diesel::dsl::max;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection = &mut establish_connection();

    let target_type_t = "TRADE".to_string();
    let mut avg_value_population: f32 = 0.0;

    let micro_var = 's';
    match find_all_with_t(connection, &target_type_t) {
        Ok(trade_objects) => {
            for trade in &trade_objects {
                println!("Item id {:?}", trade.id);

                let ec = calculate_ec_for_one_trade(trade.id, &trade_objects);
                println!("c is {:?}", &ec);

                let result = update_partioned_table_with_ec_for_one_trade(connection, micro_var, trade.id, ec);

                println!("Column c: {:?}", result);
            }
        }
        Err(e) => {
            eprintln!("Error fetching objects: {:?}", e);
        }
    }

    let result: Result<Option<f64>, diesel::result::Error> =
        objects_s.select(avg(c)).first(connection);

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

    let result_max: Result<Option<f32>, diesel::result::Error> =
        objects_s.select(max(s)).first(connection);

    match result_max {
        Ok(Some(max_value)) => {
            println!("Whole population max s: {:?}", max_value);

            let result_2: Result<Option<f64>, diesel::result::Error> = objects_s
                .filter(s.ge(divide_s).and(s.le(max_value)))
                .select(avg(c))
                .first(connection);

            match result_2 {
                Ok(Some(average_value_2)) => {
                    println!("Population_2 c average: {:?}", average_value_2)
                }
                Ok(None) => println!("No data found to calculate the c average."),
                Err(e) => eprintln!("Error calculating c average: {:?}", e),
            }
        }
        Ok(None) => println!("No data found to calculate the max."),
        Err(e) => eprintln!("Error calculating max: {:?}", e),
    }

    use diesel::dsl::count;
    use diesel::expression_methods::AggregateExpressionMethods;

    let counts = objects_s
        .select((
            count(id),
            count(id).aggregate_filter(c.lt(avg_value_population)),
            count(id).aggregate_filter(s.lt(195000.00)),
            count(id).aggregate_filter(s.lt(195000.00).and(c.lt(avg_value_population))),
            count(id).aggregate_filter(s.ge(195000.00)),
            count(id).aggregate_filter(s.ge(195000.00).and(c.lt(avg_value_population))),
        ))
        .first::<(i64, i64, i64, i64, i64, i64)>(connection)?;

    let n: f32 = counts.0 as f32;
    println!("Whole Population count n: {:?}", n);

    let m: f32 = counts.1 as f32;
    println!("Whole population m: {:?}", m);

    let n1: f32 = counts.2 as f32;
    println!("Population 1 count n1: {:?}", n1);

    let m1: f32 = counts.3 as f32;
    println!("Population 1 m1: {:?}", m1);

    let n2: f32 = counts.4 as f32;
    println!("Population 2 count n2: {:?}", n2);

    let m2: f32 = counts.5 as f32;
    println!("Population 2 m2: {:?}", m2);

    let (_p_temp, p1, p2) = ai_prop::calculate_proportions(
        ai_prop::PopulationData { m, n },
        ai_prop::PopulationData { m: m1, n: n1 },
        ai_prop::PopulationData { m: m2, n: n2 },
    );

    let pooled_estimate = ai_prop::calculate_pooled_estimate(n1, n2, p1, p2);

    ai_prop::calculate_z_statistics(n1, n2, p1, p2, pooled_estimate);

    Ok(())
}

pub fn update_partioned_table_with_ec_for_one_trade(connection: &mut diesel::PgConnection, micro_var: char, trade_id: i32, ec: f32) -> f32 {
    if micro_var == 's' {
        if let Err(e) = diesel::update(objects_s.filter(id.eq(trade_id)))
            .set(c.eq(ec))
            .execute(connection)
        {
            eprintln!("Error updating column ce for ObjectS: {:?}", e);
        }
    }

    let result: f32 = objects_s
        .select(c)
        .filter(id.eq(trade_id))
        .first(connection)
        .expect("Error loading object ec");

    result
}
