use ai_infra::schema::objects_s::dsl::*;
use diesel::PgConnection;
use diesel::dsl::{max, min};
use diesel::prelude::*;
use std::io::{self, Write};

pub fn prompt_microstructure_variable(connection: &mut PgConnection) -> Option<char> {
    print!("Choose a microstructure variable amongst the following choices: 1) Trade size: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();

    // Read user input
    loop {
        input.clear();
        let bytes = io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        if bytes == 0 {
            return None; // EOF
        }

        if !input.trim().is_empty() {
            break;
        }
    }

    // Check if input is "1)" or "1"
    let trimmed = input.trim();
    if trimmed == "1)" || trimmed == "1" {
        // Query min and max for 's' where 't' is "TRADE"
        let result: Result<Option<(Option<f32>, Option<f32>)>, _> = objects_s
            .filter(t.eq("TRADE"))
            .select((min(s), max(s)))
            .first(connection)
            .optional();

        match result {
            Ok(Some((Some(min_val), Some(max_val)))) => {
                println!(
                    "Range of values for trade size: min = {}, max = {}",
                    min_val, max_val
                );
            }
            _ => {
                println!("Could not find min/max values for trade size.");
            }
        }
        return Some('s');
    } else {
        // Anything else will default to exiting the function
        return None;
    }
}

pub fn prompt_cutoff_value() -> Option<f32> {
    print!(
        "Enter a cutoff value for the microstructure variable you selected, it should be within the range: "
    );
    io::stdout().flush().unwrap();

    let mut input = String::new();

    // Read user input
    loop {
        input.clear();
        let bytes = io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        if bytes == 0 {
            return None; // EOF
        }

        if !input.trim().is_empty() {
            break;
        }
    }

    // The user can only enter a number for now, anything else will exit the function.
    if let Ok(divide) = input.trim().parse::<f32>() {
        // The function should store the cutoff value in the variable called divide.
        Some(divide)
    } else {
        // anything else will exit the function
        None
    }
}
