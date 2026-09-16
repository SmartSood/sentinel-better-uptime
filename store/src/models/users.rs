use std::string;

use crate::{ store::Store};
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]

struct User{
    id: String,
    email: String,
    password: String
}

impl Store{
    pub fn sign_up(&mut self, email: String, password: String)-> Result<String, diesel::result::Error>{
        let id=uuid::Uuid::new_v4();
        let u=User{
            id: id.to_string(),
            email,
            password
        };
        diesel::insert_into(crate::schema::users::table)
         .values(u)
         .returning(User::as_returning())
         .get_result(&mut self.conn)?;

        Ok(id.to_string())

    }

    pub fn sign_in(&mut self, input_email:String, input_password: String) -> Result<String, diesel::result::Error>{
        use crate::schema::users::dsl::*;
        let user_result=users
        .filter(email.eq(input_email))
        .select(User::as_select())
        .first(&mut self.conn)?;


        if user_result.password==input_password {
            Ok(user_result.id)
        }
        else{
            Err(diesel::result::Error::NotFound)
        }
    }

}