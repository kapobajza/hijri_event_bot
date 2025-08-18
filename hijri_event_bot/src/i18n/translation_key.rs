#[derive(Debug, Clone)]
pub enum TranslationKey {
    Help,
    CurrentHijriDate,
    WelcomeMessage,
    WhiteDaysNotification,

    // Error keys
    ErrorGeneral,
    ErrorScheduleWhiteDaysMessage,
    ErrorCurrentDate,

    // Months
    MonthMuharram,
    MonthSafar,
    MonthRabiAlAwwal,
    MonthRabiAlThani,
    MonthJumadaAlAwwal,
    MonthJumadaAlThani,
    MonthRajab,
    MonthShaaban,
    MonthRamadan,
    MonthShawwal,
    MonthDhuAlQiDah,
    MonthDhuAlHijjah,
    MonthUnknown,
}

impl From<&TranslationKey> for &str {
    fn from(value: &TranslationKey) -> Self {
        match value {
            TranslationKey::Help => "help",
            TranslationKey::CurrentHijriDate => "current_hijri_date",
            TranslationKey::WelcomeMessage => "welcome_message",
            TranslationKey::WhiteDaysNotification => "white_days_notification",

            // Error keys
            TranslationKey::ErrorGeneral => "error_general",
            TranslationKey::ErrorScheduleWhiteDaysMessage => "error_schedule_white_days_message",
            TranslationKey::ErrorCurrentDate => "error_current_date",

            // Months
            TranslationKey::MonthMuharram => "month_muharram",
            TranslationKey::MonthSafar => "month_safar",
            TranslationKey::MonthRabiAlAwwal => "month_rabi_al_awwal",
            TranslationKey::MonthRabiAlThani => "month_rabi_al_thani",
            TranslationKey::MonthJumadaAlAwwal => "month_jumada_al_awwal",
            TranslationKey::MonthJumadaAlThani => "month_jumada_al_thani",
            TranslationKey::MonthRajab => "month_rajab",
            TranslationKey::MonthShaaban => "month_shaaban",
            TranslationKey::MonthRamadan => "month_ramadan",
            TranslationKey::MonthShawwal => "month_shawwal",
            TranslationKey::MonthDhuAlQiDah => "month_dhu_al_qi_dah",
            TranslationKey::MonthDhuAlHijjah => "month_dhu_al_hijjah",
            TranslationKey::MonthUnknown => "month_unknown",
        }
    }
}
