use crate::errors::AppError;

pub fn get_env_vars<T>(key: String) -> Result<T, AppError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let value = std::env::var(&key).map_err(|_| AppError::MissingEnvironmentVarible(key))?;
    value
        .parse::<T>()
        .map_err(|err| AppError::ParsingError(err.to_string()))
}
