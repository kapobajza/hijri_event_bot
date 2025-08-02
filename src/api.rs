use std::sync::Arc;

use serde::{Deserialize, de::DeserializeOwned};

use crate::{error::AppError, i18n::I18n};

#[derive(Deserialize)]
pub struct HijriMonth {
    pub number: u8,
    pub ar: String,
}

#[derive(Deserialize)]
pub struct HijriDate {
    pub day: String,
    pub month: HijriMonth,
    pub year: String,
}

#[derive(Deserialize)]
pub struct HijriData {
    pub hijri: HijriDate,
}

#[derive(Deserialize)]
pub struct HijriApiResponse {
    pub data: HijriData,
}

fn pad_left(value: &str, width: usize) -> String {
    let mut padded = value.to_string();
    while padded.len() < width {
        padded.insert(0, '0');
    }
    padded
}

pub struct CurrentDateResponse {
    pub day: String,
    pub day_number: u8,
    pub month: String,
    pub year: String,
    pub month_name: String,
    pub month_ar: String,
}

impl CurrentDateResponse {
    fn map_translated_month(month: u8, i18n: &I18n) -> String {
        match month {
            1 => i18n.t("month_muharram"),
            2 => i18n.t("month_safar"),
            3 => i18n.t("month_rabi_al_awwal"),
            4 => i18n.t("month_rabi_al_thani"),
            5 => i18n.t("month_jumada_al_awwal"),
            6 => i18n.t("month_jumada_al_thani"),
            7 => i18n.t("month_rajab"),
            8 => i18n.t("month_shaaban"),
            9 => i18n.t("month_ramadan"),
            10 => i18n.t("month_shawwal"),
            11 => i18n.t("month_dhu_al_qi_dah"),
            12 => i18n.t("month_dhu_al_hijjah"),
            _ => i18n.t("month_unknown"),
        }
    }

    pub fn new(hijri_data: HijriApiResponse, i18n: &I18n) -> Self {
        Self {
            day: pad_left(&hijri_data.data.hijri.day, 2),
            month: pad_left(&hijri_data.data.hijri.month.number.to_string(), 2),
            year: hijri_data.data.hijri.year,
            month_name: CurrentDateResponse::map_translated_month(
                hijri_data.data.hijri.month.number,
                i18n,
            ),
            month_ar: hijri_data.data.hijri.month.ar,
            day_number: hijri_data.data.hijri.day.parse().unwrap_or(0),
        }
    }
}

impl std::fmt::Display for CurrentDateResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}-{} ({})",
            self.year, self.month, self.day, self.month_name
        )
    }
}

pub struct HijriApi {
    api_url: String,
    i18n: Arc<I18n>,
}

impl HijriApi {
    pub fn new(i18n: Arc<I18n>) -> Self {
        Self {
            api_url: "https://api.aladhan.com/v1".to_string(),
            i18n,
        }
    }

    async fn do_request<T>(&self, route: &str, error_translation_key: &str) -> Result<T, AppError>
    where
        T: DeserializeOwned,
    {
        let response = reqwest::get(format!("{}{}", self.api_url, route))
            .await
            .map_err(|err| AppError::new(err.to_string(), error_translation_key))?;

        if !response.status().is_success() {
            return Err(AppError::new(
                format!(
                    "Failed to fetch data: {} - {}",
                    response.status(),
                    response.text().await.unwrap_or("Unknown error".to_string())
                ),
                error_translation_key,
            ));
        }

        response.json::<T>().await.map_err(|err| {
            log::error!("Failed to parse response: {}", err);
            AppError::new(err.to_string(), error_translation_key)
        })
    }

    pub async fn get_current_hijri_date(&self) -> Result<CurrentDateResponse, AppError> {
        let date_now = chrono::Utc::now();

        let hijri_data = self
            .do_request::<HijriApiResponse>(
                &format!("/gToH/{}", date_now.format("%d-%m-%Y")),
                "error_current_date",
            )
            .await?;

        Ok(CurrentDateResponse::new(hijri_data, &self.i18n))
    }
}
