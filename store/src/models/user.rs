

use crate::store::Store;
use diesel::{dsl::exists, prelude::*};
use uuid::Uuid;


#[derive(Queryable,Insertable,Selectable)]
#[diesel(table_name = crate::schema::user)]

#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User{
    pub id: String,
    pub email: String,
    pub password: String
}






impl Store{
  

    pub fn sign_in(&mut self, input_email:String,input_password:String)-> Result<String,diesel::result::Error> {
        use crate::schema::user::dsl::*;
        let user_result=crate::schema::user::table
        .filter(email.eq(input_email))
        .select(User::as_select())
        .first::<User>(&mut self.conn)?;
        
        if(user_result.password==input_password){
            Ok(user_result.id)
        } else {
            Err(diesel::result::Error::NotFound)
        }
    }
    pub fn sign_up(&mut self, email:String,password:String)-> Result<String,diesel::result::Error> {

        let id=Uuid::new_v4();
        let u=User{
            id: id.to_string(),
            email,
            password
        };

        let exists= diesel::select(exists(crate::schema::user::table.filter(crate::schema::user::email.eq(&u.email))))
        .get_result::<bool>(&mut self.conn)?;
        if exists {
            return Err(diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, Box::new("Email already exists".to_string())));
        }


        diesel::insert_into(crate::schema::user::table)
        .values(&u)
        .returning(crate::schema::user::id)
        .get_result::<String>(&mut self.conn)?;

    Ok(id.to_string())
    }

    
}

