
use ::ai_micro::*;
use ai_infra::establish_connection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection = &mut establish_connection();

    let micro_var = if let Some(v) = helpers::prompt_microstructure_variable(connection) {
        v
    } else {
        return Ok(());
    };

    let avg_value_population =
        calculate_whole_population_trades_ec_average(connection, micro_var);

    let divide_s = if let Some(cutoff) = helpers::prompt_cutoff_value() {
        cutoff
    } else {
        return Ok(());
    };
    let _divide = divide::Divide::Float(divide_s);

    // Something else
    calculate_population_1_trades_ec_average(connection, divide::Divide::Float(divide_s), micro_var);

    calculate_population_2_trades_ec_average(connection, divide::Divide::Float(divide_s), micro_var);

    let (p1, p2, n1, n2) = calculate_proportions_partioned_table(connection, avg_value_population)?;

    let pooled_estimate = ai_prop::calculate_pooled_estimate(n1, n2, p1, p2);

    ai_prop::calculate_z_statistics(n1, n2, p1, p2, pooled_estimate);

    Ok(())
}
