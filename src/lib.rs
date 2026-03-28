use ai_infra::models::ObjectS;
use ai_infra::schema::objects_s::dsl::objects_s;
use ai_infra::schema::objects_s::*;
use diesel::dsl::avg;
use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl, RunQueryDsl};

pub fn backwards(
    connection: &mut PgConnection,
    start_object_id: i32,
    target_type: &String,
) -> Result<Vec<ObjectS>, diesel::result::Error> {
    let result_vector = objects_s
        .filter(id.le(start_object_id))
        .order(id.desc())
        .limit(100)
        .select(ObjectS::as_select())
        .load::<ObjectS>(connection)?
        .into_iter()
        .find(|backward_item| backward_item.t == *target_type)
        .into_iter()
        .collect();

    Ok(result_vector)
}

pub fn calculate_mp(ap: f32, bp: f32) -> f32 {
    (ap + bp) / 2.0
}

pub fn calculate_c(pt: f32, mp: f32) -> f32 {
    (pt - mp).abs()
}

pub fn find_all_with_t(
    connection: &mut PgConnection,
    target_type: &String,
) -> QueryResult<Vec<ObjectS>> {
    objects_s
        .filter(t.eq(target_type))
        .limit(1000)
        .load::<ObjectS>(connection)
}

pub fn c_average(connection: &mut PgConnection) -> Result<Option<f64>, diesel::result::Error> {
    objects_s.select(avg(c)).first(connection)
}

pub fn calculate_ec_for_one_trade(start_object_id: i32, trade_objects: &Vec<ObjectS>) -> f32 {
    let mut connection = ai_infra::establish_connection();
    let target_type_a = "ASK".to_string();
    let target_type_b = "BID".to_string();
    let mut ap: f32 = 0.0;
    let mut bp: f32 = 0.0;
    let mut pt: f32 = 0.0;

    if let Some(trade) = trade_objects.iter().find(|o| o.id == start_object_id) {
        pt = trade.p;
    }

    if let Some(p_val) = find_nearest(&mut connection, start_object_id, &target_type_a) {
        ap = p_val;
    }

    if let Some(p_val) = find_nearest(&mut connection, start_object_id, &target_type_b) {
        bp = p_val;
    }

    let mp = calculate_mp(ap, bp);
    calculate_c(pt, mp)
}

pub fn calculate_whole_population_trades_ec_average(
    connection: &mut PgConnection,
    micro_var: char,
) -> f32 {
    let target_type_t = "TRADE".to_string();
    let mut avg_value_population: f32 = 0.0;

    match find_all_with_t(connection, &target_type_t) {
        Ok(trade_objects) => {
            for trade in &trade_objects {
                println!("Item id {:?}", trade.id);

                let ec = calculate_ec_for_one_trade(trade.id, &trade_objects);
                println!("c is {:?}", &ec);

                let result = update_partioned_table_with_ec_for_one_trade(
                    connection, micro_var, trade.id, ec,
                );

                println!("Column c: {:?}", result);
            }
        }
        Err(e) => {
            eprintln!("Error fetching objects: {:?}", e);
        }
    }

    if micro_var == 's' {
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
    }

    avg_value_population
}

pub fn update_partioned_table_with_ec_for_one_trade(
    connection: &mut diesel::PgConnection,
    micro_var: char,
    trade_id: i32,
    ec: f32,
) -> f32 {
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

pub fn find_nearest(
    connection: &mut PgConnection,
    start_object_id: i32,
    target_type: &String,
) -> Option<f32> {
    let mut p_val = None;
    match backwards(connection, start_object_id, target_type) {
        Ok(items) => {
            if let Some(item) = items
                .into_iter()
                .filter(|i| i.t == *target_type)
                .max_by_key(|i| i.d)
            {
                println!(
                    "Found object: id={:?}, type={:?}, date={:?}",
                    item.id, item.t, item.d
                );
                p_val = Some(item.p);
            }
        }
        Err(e) => {
            eprintln!("Error fetching objects: {}", e);
        }
    }
    p_val
}
