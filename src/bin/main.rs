use core::f32;

use ::ai_micro::*;
use ai_infra::establish_connection;
use ai_infra::schema::objects_s::dsl::objects_s;
use ai_infra::schema::objects_s::*;
use diesel::dsl::max;
use diesel::{QueryDsl, RunQueryDsl};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection = &mut establish_connection();

    let micro_var = 's';
    let avg_value_population =
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

    let (p1, p2, n1, n2) = calculate_proportions_partioned_table(connection, avg_value_population)?;

    let pooled_estimate = ai_prop::calculate_pooled_estimate(n1, n2, p1, p2);

    ai_prop::calculate_z_statistics(n1, n2, p1, p2, pooled_estimate);

    Ok(())
}
