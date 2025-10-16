use diesel::{RunQueryDsl, QueryDsl, SelectableHelper, ExpressionMethods};
use infra::establish_connection;
use infra::schema::objects_s::dsl::objects_s;
use infra::schema::objects_s::*;
use infra::models::ObjectS;
use::micro::*;

fn main() {
    let connection = &mut establish_connection();

    let target_type_a = "ASK".to_string();
    let target_type_b = "BID".to_string();
    let target_type_t = "TRADE".to_string();
    let mut ap = 0.0;
    let mut bp = 0.0;
    let pt = 0.0;

    match find_all_with_t(connection, &target_type_t) {
        Ok(trade_objects) => {
            for item in trade_objects {
                println!("Item id {:?}", item.id);
                let start_object_id = item.id;

                match micro::backwards(connection, start_object_id, &target_type_a) {
                    Ok(items) => {
                        for item in items {
                            println!("Found object: id={:?}, type={:?}, date={:?}", item.id, item.t, item.d);
                            ap = item.p;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error fetching objects: {}", e);
                    }
                }

                match micro::backwards(connection, start_object_id, &target_type_b) {
                    Ok(items) => {
                        for item in items {
                            println!("Found object: id={:?}, type={:?}, date={:?}", item.id, item.t, item.d);                
                            bp = item.p;
                         }
                    }
                    Err(e) => {
                        eprintln!("Error fetching objects: {}", e);
                    }
                }

                let mp = calculate_mp(&ap, &bp);
                let ec = calculate_c(&pt, &mp);       
                println!("c is {:?}", &ec);

                diesel::update(objects_s.filter(id.eq(start_object_id)))
                    .set(c.eq(ec))
                    .returning(ObjectS::as_select())
                    .get_result(connection)
                    .expect("Error updating column ce for ObjectS");

                let result: f32 = objects_s
                    .select(c) 
                    .filter(id.eq(start_object_id))
                    .first(connection) 
                    .expect("Error loading object ec");

                println!("Column c: {:?}", result);
            }
        }
        Err(e) => {
            eprintln!("Error fetching objects: {:?}", e);
        }
    }
} 
