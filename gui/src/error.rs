use house_server::error::HouseExchangeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GuiError {
    #[error("House error: {0}")]
    Exchange(#[from] HouseExchangeError),

    #[error("Iced error: {0}")]
    Iced(#[from] iced::Error),
}
