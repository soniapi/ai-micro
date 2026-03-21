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

    let mut result_vector: Vec<ObjectS> = Vec::new();

    let query_results = objects_s
        .filter(id.le(start_object_id))
        .order(id.desc())
        .limit(100)
        .select(ObjectS::as_select())
        .load(connection)?;

    for backward_item in query_results {
        if backward_item.t == *target_type {
            result_vector.push(backward_item);
            break;
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_mp() {
        assert_eq!(calculate_mp(&0.0, &0.0), 0.0);
        assert_eq!(calculate_mp(&10.0, &20.0), 15.0);
        assert_eq!(calculate_mp(&-10.0, &10.0), 0.0);
        assert_eq!(calculate_mp(&-20.0, &-10.0), -15.0);
        assert_eq!(calculate_mp(&1.5, &2.5), 2.0);
    }

    #[test]
    fn test_calculate_c() {
        assert_eq!(calculate_c(&10.0, &5.0), 5.0);
        assert_eq!(calculate_c(&5.0, &10.0), 5.0);
        assert_eq!(calculate_c(&0.0, &0.0), 0.0);
        assert_eq!(calculate_c(&-5.0, &-10.0), 5.0);
        assert_eq!(calculate_c(&-10.0, &-5.0), 5.0);
        assert_eq!(calculate_c(&5.0, &-5.0), 10.0);
    }
}