use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex, UniqueIndex};

use cw_controllers::{Admin, Hooks};

pub const ADMIN: Admin = Admin::new("admin");
pub const BLACKLIST: Hooks = Hooks::new("blacklist");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
pub enum GameMove {
    Rock {},
    Paper {},
    Scissors {},
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
pub enum GameResult {
    HostWins {},
    OpponentWins {},
    Tie {},
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
pub struct Game {
    pub host: Addr,
    pub opponent: Addr,
    pub host_move: GameMove,
    // optional, not known at the start of the game
    pub opponent_move: Option<GameMove>,
    // optional, not known before opponent_move is known
    pub result: Option<GameResult>,
}

// INDEXED MAP

pub struct GameIndexes<'a> {
    // TODO: decide which approach is needed here
    // for host, the pkey is specified as (Addr,Addr) tuple
    // so that the lookup can return vector like [((Addr,Addr), Game)]
    pub host: MultiIndex<'a, Addr, Game, (Addr, Addr)>,
    // without specifying pkey type, the returned vector is like [((), Game)]
    // which in next step can be mapped to just [Game, Game,...] - is the key needed at any point?
    pub opponent: MultiIndex<'a, Addr, Game>,
    // TODO: is this index needed?
    pub host_opponent_id: UniqueIndex<'a, (Addr, Addr), Game>,
}

// this may become a macro, not important just boilerplate, builds the list of indexes for later use
impl<'a> IndexList<Game> for GameIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Game>> + '_> {
        let v: Vec<&dyn Index<Game>> = vec![&self.host, &self.opponent, &self.host_opponent_id]; // only adjustment here, needs to list all indices
        Box::new(v.into_iter())
    }
}

pub fn games<'a>() -> IndexedMap<'a, (Addr, Addr), Game, GameIndexes<'a>> {
    let indexes = GameIndexes {
        host: MultiIndex::new(
            |d| d.host.clone(),
            "games",
            "game__host"
        ),
        opponent: MultiIndex::new(
            |d| d.opponent.clone(),
            "games",
            "game__opponent"
        ),
        host_opponent_id: UniqueIndex::new(
            // idx_fn that creates tuple key for Game
            |d| (d.host.clone(), d.opponent.clone()),
            "game__host_opponent_id"
        )
    };
    IndexedMap::new("games", indexes)
}
