use chrono::Utc;

use crate::{
    domain::{main_class::MainClass, trade::Trade},
    repositories::{main_class_repo::MainClassRepo, trade_repo::TradeRepo},
    services::{
        main_class_service::MainClassService, sector_service::SectorService,
        trade_service::TradeService,
    },
};

pub struct TradeController<'a> {
    trade_repo: &'a TradeRepo,
    main_class_repo: &'a MainClassRepo,
}

impl<'a> TradeController<'a> {
    pub fn new(trade_repo: &'a TradeRepo, main_class_repo: &'a MainClassRepo) -> Self {
        Self {
            trade_repo,
            main_class_repo,
        }
    }

    /// Create a trade and automatically create main classes for each level in [class_min..=class_max].
    pub async fn create_trade_and_main_classes(
        &self,
        new_trade: Trade,
        sector_service: &SectorService<'a>,
    ) -> Result<(Trade, Vec<MainClass>), String> {
        let trade_service = TradeService::new(self.trade_repo);
        let main_class_service = MainClassService::new(self.main_class_repo, &trade_service);

        let inserted_trade = trade_service
            .create_trade(new_trade, sector_service)
            .await?;

        let trade_oid = inserted_trade
            .id
            .ok_or_else(|| "Inserted trade has no id".to_string())?;

        let mut min = inserted_trade.class_min;
        let mut max = inserted_trade.class_max;
        if min > max {
            std::mem::swap(&mut min, &mut max);
        }

        if max < min {
            return Ok((inserted_trade, vec![]));
        }

        // ✅ Now works because TradeType implements Display
        let trade_type_name = inserted_trade.r#type.to_string();

        let mut main_classes_to_create = Vec::new();
        for level in min..=max {
            let name = format!("{} {} {}", trade_type_name, level, inserted_trade.name);
            let username = format!(
                "{}_{}_{}",
                trade_type_name.to_lowercase(),
                level,
                inserted_trade.username.replace(' ', "_").to_lowercase()
            );

            main_classes_to_create.push(MainClass {
                id: None,
                name,
                username,
                trade_id: Some(trade_oid),
                level: Some(level),
                description: Some(format!("Auto-created for trade {}", inserted_trade.name)),
                disable: Some(false),
                created_at: Some(Utc::now()),
                updated_at: Some(Utc::now()),
            });
        }

        let created_main_classes = if main_classes_to_create.is_empty() {
            vec![]
        } else {
            main_class_service
                .create_many_main_classes(main_classes_to_create)
                .await?
        };

        Ok((inserted_trade, created_main_classes))
    }
}
