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
        time: NaiveDateTime,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO (user_id, course_id, purchase_time) payment_history VALUES (?, ?, ?)",
        )
        .bind(user_id)
        .bind(course_id)
        .bind(time)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// # Получение нового экземпляра
    /// Назвал не new, потому что Result
    pub async fn get_pool() -> Result<Self, sqlx::Error> {
        Ok(Self {
            pool: MySqlPool::connect("mariadb://root:root@localhost/teoneo").await?,
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
