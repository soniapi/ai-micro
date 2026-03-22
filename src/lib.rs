use diesel::dsl::avg;
use diesel::{PgConnection, RunQueryDsl, QueryDsl};
use diesel::prelude::*;
use ai_infra::models::ObjectS;
use ai_infra::schema::objects_s::*;
use ai_infra::schema::objects_s::dsl::objects_s;

pub fn backwards (
    connection: &mut PgConnection,
    start_object_id: i32,
    target_type: &String,
) -> Result<Vec<ObjectS>, diesel::result::Error> {

    objects_s
        .filter(id.le(start_object_id))
        .filter(t.eq(target_type))
        .order(id.desc())
        .limit(1)
        .select(ObjectS::as_select())
        .load(connection)?;

    let result_vector = query_results
        .into_iter()
        .find(|backward_item| backward_item.t == *target_type)
        .into_iter()
        .collect();

    Ok(result_vector)
}

pub fn calculate_mp (ap: &f32, bp: &f32) -> f32 {
    ( ap + bp ) / 2.0
}

pub fn calculate_c (pt: &f32, mp: &f32) -> f32 {
    (pt - mp).abs()
}

pub fn find_all_with_t (connection: &mut PgConnection, target_type: &String) -> QueryResult<Vec<ObjectS>> {
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