use crate::prelude::*;
use actix_web::web;
use schema::*;

pub enum BookType {
    PAPER = 1, // 纸书
    PDF = 2,   // PDF
    TXT = 3,   // TXT
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BookWithUser<A, B> {
    #[serde(flatten)]
    book: A,
    user: B,
}
impl<A, B> From<(A, B)> for BookWithUser<A, B> {
    fn from((book, user): (A, B)) -> Self {
        Self { book, user }
    }
}

#[derive(Identifiable, Associations, Queryable, Serialize, Deserialize, Debug)]
#[diesel(belongs_to(models::user::User, foreign_key = user_id))]
#[diesel(table_name = books)]
pub struct Book {
    pub id: PK,
    pub name: String,
    pub price: BigDecimal,
    pub user_id: PK,
    #[serde(rename(serialize = "type"))]
    pub type_: u8,
    pub stock: u32,
    pub create_at: NaiveDateTime,
    pub update_at: NaiveDateTime,
}

impl Book {
    pub async fn create(db_pool: web::Data<DbPool>, data: BookNew) -> R<usize> {
        let new_rows: usize = web::block(move || -> R<_> {
            let mut conn = conn!(db_pool);
            Ok(diesel::insert_into(books::table)
                .values(&data)
                .execute(&mut conn)?)
        })
        .await??;

        Ok(new_rows)
    }

    pub async fn obj(db_pool: web::Data<DbPool>, pk: PK) -> R<Book> {
        let res: Book = web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            Ok(books::table.find(pk).first(&mut conn)?)
        })
        .await??;

        Ok(res)
    }

    pub async fn del(db_pool: web::Data<DbPool>, pk: PK) -> R<usize> {
        let rows: usize = web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            Ok(diesel::delete(books::table)
                .filter(books::id.eq(pk))
                .execute(&mut conn)?)
        })
        .await??;

        Ok(rows)
    }

    pub async fn update(db_pool: web::Data<DbPool>, pk: PK, data: BookUpdate) -> R<usize> {
        let rows: usize = web::block(move || -> R<_> {
            let mut conn = db_pool.get()?;
            let target = books::table.find(pk);
            Ok(diesel::update(target).set(&data).execute(&mut conn)?)
        })
        .await??;

        Ok(rows)
    }
}

#[derive(Insertable, Validate, Deserialize, Debug)]
#[diesel(table_name = books)]
pub struct BookNew {
    pub name: String,
    pub price: BigDecimal,
    pub user_id: PK,
    #[serde(rename(deserialize = "type"))]
    pub type_: u8,
    pub stock: u32,
}

#[derive(AsChangeset, Identifiable, Validate, Deserialize, Debug)]
#[diesel(table_name = books)]
pub struct BookUpdate {
    pub id: PK,
    pub name: Option<String>,
    pub user_id: Option<PK>,
    pub type_: Option<u8>,
    pub price: Option<BigDecimal>,
    pub stock: Option<u32>,
}
