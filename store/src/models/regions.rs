use crate::store::Store;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::region)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Region{
    pub  id:String,
    pub name:String,

}
pub struct RegionsOutput{
    pub regions: Vec<Region>
}
impl Store{
    pub fn get_all_regions(&mut self) -> Result<Vec<Region>, diesel::result::Error> {
        use crate::schema::region::dsl::*;
        let region_result=crate::schema::region::table
        .select(Region::as_select())
        .load::<Region>(&mut self.conn)?;

        Ok(region_result)
    }
}