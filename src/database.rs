use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use sqlx::{
    mysql::{MySqlPool, MySqlRow},
    Row,
};

/// # Структура для представления записей из таблицы
/// Содержит поля, каждое из которых соответствует полю из таблицы
#[derive(Debug)]
pub struct Payment {
    id: i64,
    user_id: i64,
    course_id: i64,
    purchase_time: NaiveDateTime,
}

impl Payment {
    /// Конструирует методы из результата запроса
    pub fn from_row(row: MySqlRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.get("id"),
            user_id: row.get("user_id"),
            course_id: row.get("course_id"),
            purchase_time: row.get("purchase_time"),
        })
    }
}

/// Структура для пула
#[derive(Clone)]
pub struct PaymentDB {
    pool: MySqlPool,
}

impl PaymentDB {
    /// # Добавление записи в бд
    /// Каждый параметр соответствует полю из бд, id не передается, т.к. автоинкремент
    pub async fn register_payment(
        self,
        user_id: i64,
        course_id: i64,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = self.pool.begin().await?;
        // Эти 2 запроса отправляют atomicaly (все или ничего), чтобы это сделать в mySql, нужно использовать транзакции
        // Если один из запросов фейлится то на drop() вызывается rollback() и изменения отменяются

        sqlx::query(
            "INSERT INTO payment_history (user_id, course_id) VALUES (?, ?)",
        )
        .bind(user_id)
        .bind(course_id)
        .execute(&mut *transaction)
        .await?; 
        // payments_history используется как чек, где будут и успешные оплаты и не успешные
        sqlx::query("INSERT INTO user_courses (user_id, course_id) VALUES (?, ?)")
            .bind(user_id)
            .bind(course_id)
            .execute(&mut *transaction)
            .await?;

        transaction.commit().await?;
        Ok(())
    }

    /// # Получение нового экземпляра
    /// Назвал не new, потому что Result
    pub async fn get_pool() -> Result<Self, sqlx::Error> {
        Ok(Self {
            pool: MySqlPool::connect(&std::env::var("DATABASE_URL").expect("No database url variable in .env")).await?,
        })
    }

    /// Получение информации об операции по айдишнику
    pub async fn get_payment_info(&self, id: i64) -> Result<Payment, sqlx::Error> {
        Ok(Payment::from_row(
            sqlx::query("SELECT * FROM payment_history WHERE id=?")
                .bind(id)
                .fetch_one(&self.pool)
                .await?,
        )?)
    }

    pub async fn get_course_price(&self, id: i64) -> Result<BigDecimal, sqlx::Error> {
        Ok(sqlx::query("SELECT price FROM courses WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?
            .get("price"))
    }
}
