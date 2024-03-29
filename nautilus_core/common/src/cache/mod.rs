// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2024 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

#![allow(dead_code)] // Under development

pub mod database;

use std::collections::{HashMap, HashSet, VecDeque};

use nautilus_core::correctness::{check_slice_not_empty, check_valid_string};
use nautilus_model::{
    data::{
        bar::{Bar, BarType},
        quote::QuoteTick,
        trade::TradeTick,
    },
    identifiers::{
        account_id::AccountId, client_id::ClientId, client_order_id::ClientOrderId,
        component_id::ComponentId, exec_algorithm_id::ExecAlgorithmId, instrument_id::InstrumentId,
        position_id::PositionId, strategy_id::StrategyId, venue::Venue,
        venue_order_id::VenueOrderId,
    },
    instruments::{synthetic::SyntheticInstrument, Instrument},
    orderbook::book::OrderBook,
    orders::base::Order,
    position::Position,
    types::currency::Currency,
};
use tracing::{debug, info};
use ustr::Ustr;

use self::database::CacheDatabaseAdapter;
use crate::enums::SerializationEncoding;

pub struct CacheConfig {
    pub encoding: SerializationEncoding,
    pub timestamps_as_iso8601: bool,
    pub use_trader_prefix: bool,
    pub use_instance_id: bool,
    pub flush_on_start: bool,
    pub drop_instruments_on_reset: bool,
    pub tick_capacity: usize,
    pub bar_capacity: usize,
}

impl CacheConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        encoding: SerializationEncoding,
        timestamps_as_iso8601: bool,
        use_trader_prefix: bool,
        use_instance_id: bool,
        flush_on_start: bool,
        drop_instruments_on_reset: bool,
        tick_capacity: usize,
        bar_capacity: usize,
    ) -> Self {
        Self {
            encoding,
            timestamps_as_iso8601,
            use_trader_prefix,
            use_instance_id,
            flush_on_start,
            drop_instruments_on_reset,
            tick_capacity,
            bar_capacity,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self::new(
            SerializationEncoding::MsgPack,
            false,
            true,
            false,
            false,
            true,
            10_000,
            10_000,
        )
    }
}

pub struct CacheIndex {
    venue_account: HashMap<Venue, AccountId>,
    venue_orders: HashMap<Venue, HashSet<ClientOrderId>>,
    venue_positions: HashMap<Venue, HashSet<PositionId>>,
    order_ids: HashMap<VenueOrderId, ClientOrderId>,
    order_position: HashMap<ClientOrderId, PositionId>,
    order_strategy: HashMap<ClientOrderId, StrategyId>,
    order_client: HashMap<ClientOrderId, ClientId>,
    position_strategy: HashMap<PositionId, StrategyId>,
    position_orders: HashMap<PositionId, HashSet<ClientOrderId>>,
    instrument_orders: HashMap<InstrumentId, HashSet<ClientOrderId>>,
    instrument_positions: HashMap<InstrumentId, HashSet<PositionId>>,
    strategy_orders: HashMap<StrategyId, HashSet<ClientOrderId>>,
    strategy_positions: HashMap<StrategyId, HashSet<PositionId>>,
    exec_algorithm_orders: HashMap<ExecAlgorithmId, HashSet<ClientOrderId>>,
    exec_spawn_orders: HashMap<ExecAlgorithmId, HashSet<ClientOrderId>>,
    orders: HashSet<ClientOrderId>,
    orders_open: HashSet<ClientOrderId>,
    orders_closed: HashSet<ClientOrderId>,
    orders_emulated: HashSet<ClientOrderId>,
    orders_inflight: HashSet<ClientOrderId>,
    orders_pending_cancel: HashSet<ClientOrderId>,
    positions: HashSet<PositionId>,
    positions_open: HashSet<PositionId>,
    positions_closed: HashSet<PositionId>,
    actors: HashSet<ComponentId>,
    strategies: HashSet<StrategyId>,
    exec_algorithms: HashSet<ExecAlgorithmId>,
}

impl CacheIndex {
    /// Clear the index which will clear/reset all internal state.
    pub fn clear(&mut self) {
        self.venue_account.clear();
        self.venue_orders.clear();
        self.venue_positions.clear();
        self.order_ids.clear();
        self.order_position.clear();
        self.order_strategy.clear();
        self.order_client.clear();
        self.position_strategy.clear();
        self.position_orders.clear();
        self.instrument_orders.clear();
        self.instrument_positions.clear();
        self.strategy_orders.clear();
        self.strategy_positions.clear();
        self.exec_algorithm_orders.clear();
        self.exec_spawn_orders.clear();
        self.orders.clear();
        self.orders_open.clear();
        self.orders_closed.clear();
        self.orders_emulated.clear();
        self.orders_inflight.clear();
        self.orders_pending_cancel.clear();
        self.positions.clear();
        self.positions_open.clear();
        self.positions_closed.clear();
        self.actors.clear();
        self.strategies.clear();
        self.exec_algorithms.clear();
    }
}

pub struct Cache {
    config: CacheConfig,
    index: CacheIndex,
    database: Option<CacheDatabaseAdapter>,
    general: HashMap<String, Vec<u8>>,
    quote_ticks: HashMap<InstrumentId, VecDeque<QuoteTick>>,
    trade_ticks: HashMap<InstrumentId, VecDeque<TradeTick>>,
    order_books: HashMap<InstrumentId, OrderBook>,
    bars: HashMap<BarType, VecDeque<Bar>>,
    bars_bid: HashMap<BarType, Bar>,
    bars_ask: HashMap<BarType, Bar>,
    currencies: HashMap<Ustr, Currency>,
    instruments: HashMap<InstrumentId, Box<dyn Instrument>>,
    synthetics: HashMap<InstrumentId, SyntheticInstrument>,
    // accounts: HashMap<AccountId, Box<dyn Account>>,  TODO: Decide where trait should go
    orders: HashMap<ClientOrderId, Box<dyn Order>>, // TODO: Efficency (use enum)
    // order_lists: HashMap<OrderListId, VecDeque<OrderList>>,  TODO: Need `OrderList`
    positions: HashMap<PositionId, Position>,
    position_snapshots: HashMap<PositionId, Vec<u8>>,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new(CacheConfig::default(), None)
    }
}

impl Cache {
    pub fn new(config: CacheConfig, database: Option<CacheDatabaseAdapter>) -> Self {
        let index = CacheIndex {
            venue_account: HashMap::new(),
            venue_orders: HashMap::new(),
            venue_positions: HashMap::new(),
            order_ids: HashMap::new(),
            order_position: HashMap::new(),
            order_strategy: HashMap::new(),
            order_client: HashMap::new(),
            position_strategy: HashMap::new(),
            position_orders: HashMap::new(),
            instrument_orders: HashMap::new(),
            instrument_positions: HashMap::new(),
            strategy_orders: HashMap::new(),
            strategy_positions: HashMap::new(),
            exec_algorithm_orders: HashMap::new(),
            exec_spawn_orders: HashMap::new(),
            orders: HashSet::new(),
            orders_open: HashSet::new(),
            orders_closed: HashSet::new(),
            orders_emulated: HashSet::new(),
            orders_inflight: HashSet::new(),
            orders_pending_cancel: HashSet::new(),
            positions: HashSet::new(),
            positions_open: HashSet::new(),
            positions_closed: HashSet::new(),
            actors: HashSet::new(),
            strategies: HashSet::new(),
            exec_algorithms: HashSet::new(),
        };

        Self {
            config,
            index,
            database,
            general: HashMap::new(),
            quote_ticks: HashMap::new(),
            trade_ticks: HashMap::new(),
            order_books: HashMap::new(),
            bars: HashMap::new(),
            bars_bid: HashMap::new(),
            bars_ask: HashMap::new(),
            currencies: HashMap::new(),
            instruments: HashMap::new(),
            synthetics: HashMap::new(),
            // accounts: HashMap<AccountId, Box<dyn Account>>,  TODO: Decide where trait should go
            orders: HashMap::new(), // TODO: Efficency (use enum)
            // order_lists: HashMap<OrderListId, VecDeque<OrderList>>,  TODO: Need `OrderList`
            positions: HashMap::new(),
            position_snapshots: HashMap::new(),
        }
    }

    pub fn cache_general(&mut self) -> anyhow::Result<()> {
        self.general = match &self.database {
            Some(db) => db.load()?,
            None => HashMap::new(),
        };

        info!(
            "Cached {} general object(s) from database",
            self.general.len()
        );
        Ok(())
    }

    pub fn cache_currencies(&mut self) -> anyhow::Result<()> {
        self.currencies = match &self.database {
            Some(db) => db.load_currencies()?,
            None => HashMap::new(),
        };

        info!("Cached {} currencies from database", self.general.len());
        Ok(())
    }

    pub fn cache_instruments(&mut self) -> anyhow::Result<()> {
        self.instruments = match &self.database {
            Some(db) => db.load_instruments()?,
            None => HashMap::new(),
        };

        info!("Cached {} instruments from database", self.general.len());
        Ok(())
    }

    pub fn cache_synthetics(&mut self) -> anyhow::Result<()> {
        self.synthetics = match &self.database {
            Some(db) => db.load_synthetics()?,
            None => HashMap::new(),
        };

        info!(
            "Cached {} synthetic instruments from database",
            self.general.len()
        );
        Ok(())
    }

    // pub fn cache_accounts(&mut self) -> anyhow::Result<()> {
    //     self.accounts = match &self.database {
    //         Some(db) => db.load_accounts()?,
    //         None => HashMap::new(),
    //     };
    //
    //     info!(
    //         "Cached {} synthetic instruments from database",
    //         self.general.len()
    //     );
    //     Ok(())
    // }

    pub fn cache_orders(&mut self) -> anyhow::Result<()> {
        self.orders = match &self.database {
            Some(db) => db.load_orders()?,
            None => HashMap::new(),
        };

        info!("Cached {} orders from database", self.general.len());
        Ok(())
    }

    // pub fn cache_order_lists(&mut self) -> anyhow::Result<()> {
    //
    //
    //     info!("Cached {} order lists from database", self.general.len());
    //     Ok(())
    // }

    pub fn cache_positions(&mut self) -> anyhow::Result<()> {
        self.positions = match &self.database {
            Some(db) => db.load_positions()?,
            None => HashMap::new(),
        };

        info!("Cached {} positions from database", self.general.len());
        Ok(())
    }

    pub fn check_residuals(&self) {
        todo!() // Needs order query methods
    }

    pub fn clear_index(&mut self) {
        self.index.clear();
        debug!("Cleared index");
    }

    /// Reset the cache.
    ///
    /// All stateful fields are reset to their initial value.
    pub fn reset(&mut self) {
        debug!("Resetting cache");

        self.general.clear();
        self.quote_ticks.clear();
        self.trade_ticks.clear();
        self.order_books.clear();
        self.bars.clear();
        self.bars_bid.clear();
        self.bars_ask.clear();
        self.currencies.clear();
        self.synthetics.clear();
        // self.accounts.clear();  // TODO
        self.orders.clear();
        // self.order_lists.clear();  // TODO
        self.positions.clear();
        self.position_snapshots.clear();

        self.clear_index();

        info!("Reset cache");
    }

    pub fn dispose(&self) -> anyhow::Result<()> {
        if let Some(database) = &self.database {
            // TODO: Log operations in database adapter
            database.close()?
        }
        Ok(())
    }

    pub fn flush_db(&self) -> anyhow::Result<()> {
        if let Some(database) = &self.database {
            // TODO: Log operations in database adapter
            database.flush()?
        }
        Ok(())
    }

    pub fn add(&mut self, key: &str, value: Vec<u8>) -> anyhow::Result<()> {
        check_valid_string(key, stringify!(key))?;
        check_slice_not_empty(value.as_slice(), stringify!(value))?;

        self.general.insert(key.to_string(), value.clone());
        debug!("Added general '{key}'");

        if let Some(database) = &self.database {
            database.add(key.to_string(), value)?;
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> anyhow::Result<Option<&Vec<u8>>> {
        check_valid_string(key, stringify!(key))?;

        Ok(self.general.get(key))
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use rstest::*;

    use super::Cache;

    #[rstest]
    fn test_general_when_no_value() {
        let cache = Cache::default();
        let result = cache.get("A").unwrap();
        assert_eq!(result, None);
    }

    #[rstest]
    fn test_general_when_value() {
        let mut cache = Cache::default();

        let key = "A";
        let value = vec![0_u8];
        cache.add(key, value.clone()).unwrap();

        let result = cache.get(key).unwrap();
        assert_eq!(result, Some(&value));
    }
}
