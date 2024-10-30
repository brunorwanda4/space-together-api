pub type TradingResult<T> = core::result::Result<T, TradingErr>;

#[derive(Debug)]
pub enum TradingErr {
    CanNotCreateTrading,
    CanNotGetTrading,
    NotFoundTrading,
    CanNotUpdateTrading,
    CanNotCreateTradingIndex,
    CanNotDeleteTrading,
    CanChangeTradingIdIntoObjectId,
    NoFieldsToUpdate,
    TradingUsernameIsReadyExit,
    TradingCodeIsReadyExit,
    CanNotGetAllTradings,
}

impl std::fmt::Display for TradingErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradingErr::CanNotCreateTrading => write!(f, "Can not create trading"),
            TradingErr::CanNotGetTrading => write!(f, "Can not get trading information"),
            TradingErr::CanNotUpdateTrading => write!(f, "Can not update trading"),
            TradingErr::CanNotCreateTradingIndex => write!(f, "Can not create trading index"),
            TradingErr::CanNotDeleteTrading => write!(f, "Can not delete trading"),
            TradingErr::CanChangeTradingIdIntoObjectId => {
                write!(f, "Can not change trading ID into ObjectId")
            }
            TradingErr::NoFieldsToUpdate => write!(f, "No fields to update in Trading object "),
            TradingErr::NotFoundTrading => write!(f, " cannot find trading by id"),
            TradingErr::TradingUsernameIsReadyExit => {
                write!(f, " Trading username is ready to exit, try other!")
            }
            TradingErr::TradingCodeIsReadyExit => {
                write!(f, "Trade code are ready to exit, try other! ")
            }
            TradingErr::CanNotGetAllTradings => write!(f, "Can not get all tradings"),
        }
    }
}
