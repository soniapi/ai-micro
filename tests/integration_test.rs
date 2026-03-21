use ai_infra::establish_connection;
use ai_micro::find_all_with_t;
use ai_prop::calculate_proportions;

#[test]
fn test_end_to_end_sequence() {
    // 1. Establish database connection (from ai-infra)
    let connection = &mut establish_connection();

    // 2. Fetch data (from ai-micro)
    // We expect this to run cleanly. If no data is present, the vector will just be empty.
    let target_type = "TRADE".to_string();
    let objects = find_all_with_t(connection, &target_type).expect("Error querying database");

    let n = objects.len() as f32;

    // Ensure we can make a query
    println!("Found {} trade objects.", n);

    // 3. Mathematical calculation (from ai-prop)
    // Using some arbitrary values since we just want to verify integration of the function
    let m = 50.0;
    let m1 = 20.0;
    let m2 = 30.0;
    let n_total = 100.0;
    let n1 = 40.0;
    let n2 = 60.0;

    let (p_population, p1, p2) = calculate_proportions(&m, &m1, &m2, &n_total, &n1, &n2);

    // Some simple assertions
    assert_eq!(p_population, 0.5); // 50 / 100
    assert_eq!(p1, 0.5); // 20 / 40
    assert_eq!(p2, 0.5); // 30 / 60
}
