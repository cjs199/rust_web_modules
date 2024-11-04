use sqlx::types::BigDecimal;
use std::str::FromStr;

pub fn str_to_decimal(s: impl Into<String>) -> BigDecimal {
    let s_: String = s.into();
    BigDecimal::from_str(s_.as_str()).unwrap()
}

pub fn scale_down_default(decimal: BigDecimal) -> BigDecimal {
    scale_down(decimal, 3)
}

pub fn scale_down(decimal: BigDecimal, new_scale: i64) -> BigDecimal {
    decimal.with_scale(new_scale);
    decimal
}
