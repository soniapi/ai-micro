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

pub fn c_average (connection: &mut PgConnection) -> Result<Option<f64>, diesel::result::Error> {
    objects_s
        .select(avg(c))
        .first(connection)
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

pub fn find_nearest(
    connection: &mut PgConnection,
    start_object_id: i32,
    target_type: &String,
) -> Option<f32> {
    let mut p_val = None;
    match backwards(connection, start_object_id, target_type) {
        Ok(items) => {
            if let Some(item) = items.into_iter().filter(|i| i.t == *target_type).max_by_key(|i| i.d) {
                println!("Found object: id={:?}, type={:?}, date={:?}", item.id, item.t, item.d);
                p_val = Some(item.p);
            }
        }
        Err(e) => {
            eprintln!("Error fetching objects: {}", e);
        }
    }
    p_val
}
