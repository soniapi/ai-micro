use core::f32;

use ::ai_micro::*;
use ai_infra::establish_connection;
use ai_infra::schema::objects_s::dsl::objects_s;
use ai_infra::schema::objects_s::*;
use diesel::BoolExpressionMethods;
use diesel::dsl::max;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection = &mut establish_connection();

    let micro_var = 's';
    let mut avg_value_population =
        calculate_whole_population_trades_ec_average(connection, micro_var);

    let start_s = 0.0_f32;
    let divide_s = 195000.0_f32;

    // Something else
    calculate_population_1_trades_ec_average(connection, start_s, divide_s);

    let result_max: Result<Option<f32>, diesel::result::Error> =
        objects_s.select(max(s)).first(connection);

    match result_max {
        Ok(Some(max_value)) => {
            println!("Whole population max s: {:?}", max_value);

            calculate_population_2_trades_ec_average(connection, divide_s, max_value);
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
