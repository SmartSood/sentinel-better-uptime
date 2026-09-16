use crate::{schema::website_tick::region_id, store::Store};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable,Insertable,Selectable)]
#[diesel(table_name = crate::schema::website)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Website{
    pub id:String,
    pub url:String,
    pub user_id:String,
    pub time_added:chrono::NaiveDateTime,
    pub region_ids:Vec<Option<String>>,
    pub poll_time:i64

}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::website_tick)]
pub struct NewWebsiteTick {
    pub id: String,

    pub website_id: String,
    pub region_id: String,

    pub status: String,
    pub status_code: Option<i32>,

    pub response_time_ms: i32,

    pub dns_time_ms: Option<i32>,
    pub tcp_time_ms: Option<i32>,
    pub tls_time_ms: Option<i32>,
    pub ttfb_ms: Option<i32>,

    pub response_size_bytes: Option<i64>,

    pub content_valid: Option<bool>,

    pub ssl_valid: Option<bool>,
    pub ssl_days_remaining: Option<i32>,

    pub error: Option<String>,

    pub created_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::website_tick)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WebsiteTick {
    pub id: String,
    pub response_time_ms: i32,
    pub status: String,
    pub website_id: String,
    pub region_id: String,
    pub status_code: Option<i32>,
    pub dns_time_ms: Option<i32>,
    pub tcp_time_ms: Option<i32>,
    pub tls_time_ms: Option<i32>,
    pub ttfb_ms: Option<i32>,
    pub response_size_bytes: Option<i64>,
    pub content_valid: Option<bool>,
    pub ssl_valid: Option<bool>,
    pub ssl_days_remaining: Option<i32>,
    pub error: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}



impl Store{
    pub fn create_website(&mut self, current_user_id: String, url:String, region_ids: Vec<Option<String>>, poll_time: i64) -> Result<Website,diesel::result::Error> {
        
        let id=Uuid::new_v4();
        let website=Website{
            user_id:current_user_id,
            id: id.to_string(),
            url,
            time_added: chrono::Utc::now().naive_utc(),
            region_ids,
            poll_time:poll_time ,

    };
    
        diesel::insert_into(crate::schema::website::table)
        .values(&website)
        .execute(&mut self.conn)?;


        Ok(website)
    
}


    pub fn create_website_tick(
    &mut self,
    website_id: String,
    current_region_id: String,
    status: String,
    status_code: Option<i32>,

    response_time_ms: i32,

    dns_time_ms: Option<i32>,
    tcp_time_ms: Option<i32>,
    tls_time_ms: Option<i32>,
    ttfb_ms: Option<i32>,

    response_size_bytes: Option<i64>,

    content_valid: Option<bool>,

    ssl_valid: Option<bool>,
    ssl_days_remaining: Option<i32>,

    error: Option<String>,
) -> Result<(), diesel::result::Error> {

    let tick = NewWebsiteTick {
        id: Uuid::new_v4().to_string(),

        website_id,
        region_id: current_region_id,

        status,
        status_code,

        response_time_ms,

        dns_time_ms,
        tcp_time_ms,
        tls_time_ms,
        ttfb_ms,

        response_size_bytes,

        content_valid,

        ssl_valid,
        ssl_days_remaining,

        error,

        created_at: chrono::Utc::now().naive_utc(),
    };

    diesel::insert_into(crate::schema::website_tick::table)
        .values(&tick)
        .execute(&mut self.conn)?;

    Ok(())
}

    pub fn get_websites(&mut self, input_user_id: String) -> Result<Vec<Website>, diesel::result::Error> {
        use crate::schema::website::dsl::*;
        let website_result=crate::schema::website::table
        .filter(user_id.eq(input_user_id))
        .select(Website::as_select())
        .load::<Website>(&mut self.conn)?;

        Ok(website_result)
    }


    pub fn get_website_by_id(&mut self, input_website_id: String, input_user_id: String) -> Result<Website, diesel::result::Error> {
        use crate::schema::website::dsl::*;
        let website_result=crate::schema::website::table
        .filter(id.eq(input_website_id))
        .filter(user_id.eq(input_user_id))
        .select(Website::as_select())
        .first::<Website>(&mut self.conn)?;

        Ok(website_result)
    }

    pub fn get_website_ticks(
        &mut self,
        input_website_id: String,
        input_user_id: String,
        input_limit: i64,
    ) -> Result<Vec<WebsiteTick>, diesel::result::Error> {
        use crate::schema::{website, website_tick};

        website_tick::table
            .inner_join(website::table.on(website::id.eq(website_tick::website_id)))
            .filter(website::id.eq(input_website_id))
            .filter(website::user_id.eq(input_user_id))
            .select(WebsiteTick::as_select())
            .order(website_tick::created_at.desc())
            .limit(input_limit)
            .load::<WebsiteTick>(&mut self.conn)
    }
}