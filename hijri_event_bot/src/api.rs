use std::sync::Arc;

use serde::{Deserialize, de::DeserializeOwned};

use crate::{
    error::AppErrorKind,
    i18n::{instance::I18n, translation_key::TranslationKey},
};

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
    pub month_number: u8,
    pub month_ar: String,
}

impl CurrentDateResponse {
    fn map_translated_month(month: u8, i18n: &I18n) -> String {
        match month {
            1 => i18n.t(&TranslationKey::MonthMuharram),
            2 => i18n.t(&TranslationKey::MonthSafar),
            3 => i18n.t(&TranslationKey::MonthRabiAlAwwal),
            4 => i18n.t(&TranslationKey::MonthRabiAlThani),
            5 => i18n.t(&TranslationKey::MonthJumadaAlAwwal),
            6 => i18n.t(&TranslationKey::MonthJumadaAlThani),
            7 => i18n.t(&TranslationKey::MonthRajab),
            8 => i18n.t(&TranslationKey::MonthShaaban),
            9 => i18n.t(&TranslationKey::MonthRamadan),
            10 => i18n.t(&TranslationKey::MonthShawwal),
            11 => i18n.t(&TranslationKey::MonthDhuAlQiDah),
            12 => i18n.t(&TranslationKey::MonthDhuAlHijjah),
            _ => i18n.t(&TranslationKey::MonthUnknown),
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
            month_number: hijri_data.data.hijri.month.number,
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

    async fn do_request<T>(&self, route: &str) -> Result<T, AppErrorKind>
    where
        T: DeserializeOwned,
    {
        let response = reqwest::get(format!("{}{}", self.api_url, route))
            .await
            .map_err(|err| {
                log::error!("API request failed: {}", err);
                AppErrorKind::ApiRequest
            })?;

        if !response.status().is_success() {
            log::error!(
                "API request failed with status: {} for route: {}",
                response.status(),
                route
            );
            return Err(AppErrorKind::ApiRequest);
        }

        response.json::<T>().await.map_err(|err| {
            log::error!("Failed to parse response: {}", err);
            AppErrorKind::ApiRequest
        })
    }

    pub async fn get_current_hijri_date(&self) -> Result<CurrentDateResponse, AppErrorKind> {
        let date_now = chrono::Utc::now().with_timezone(&chrono_tz::Tz::Europe__Sarajevo);

        let hijri_data = self
            .do_request::<HijriApiResponse>(&format!("/gToH/{}", date_now.format("%d-%m-%Y")))
            .await?;

        Ok(CurrentDateResponse::new(hijri_data, &self.i18n))
    }
}
