pub type TradingResult<T> = core::result::Result<T, TradingErr>;

#[derive(Debug)]
pub enum TradingErr {
    CanNotCreateTrading,
    CanNotGetTrading,
    CanNotUpdateTrading,
    CanNotDeleteTrading,
}

impl std::fmt::Display for TradingErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradingErr::CanNotCreateTrading => write!(f, "Can not create trading"),
            TradingErr::CanNotGetTrading => write!(f, "Can not get trading information"),
            TradingErr::CanNotUpdateTrading => write!(f, "Can not update trading"),
            TradingErr::CanNotDeleteTrading => write!(f, "Can not delete trading"),
        }
    }
}
