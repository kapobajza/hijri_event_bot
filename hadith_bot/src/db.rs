use crate::error::AppErrorKind;

pub struct Hadith {
    #[allow(dead_code)]
    pub id: sqlx::types::Uuid,
    pub text_bos: String,
    pub transmitters_text: String,
    pub hadith_numbers: Option<String>,
    pub book_title: String,
    pub book_author: String,
}

pub struct HadithRepository {
    pool: sqlx::PgPool,
}

impl HadithRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_random_hadith_text(&self) -> Result<String, AppErrorKind> {
        let hadith = sqlx::query_as!(
            Hadith,
            r#"
            SELECT
                h.id,
                h.text_bos,
                h.transmitters_text,
                STRING_AGG(hn.value::text, ', ' ORDER BY hn.value) AS hadith_numbers,
                b.title AS book_title,
                b.author AS book_author
            FROM hadiths AS h
            LEFT JOIN hadith_numbers AS hn ON hn.hadith_id = h.id
            JOIN books AS b ON b.id = h.book_id
            WHERE LENGTH(h.text_bos) < 10000
            GROUP BY h.id, h.text_bos, h.transmitters_text, b.title, b.author
            ORDER BY RANDOM()
            LIMIT 1
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            log::error!("Failed to fetch random hadith: {}", err);
            AppErrorKind::GetRandomHadithFromDb
        })?;

        Ok(format!(
            "{}\n\n{}\n\n\n{}: {}\nHadis Broj: {}",
            hadith.transmitters_text,
            hadith.text_bos,
            hadith.book_author,
            hadith.book_title,
            hadith.hadith_numbers.unwrap_or_default()
        ))
    }
}
