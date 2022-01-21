use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};

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
pub struct GameData {
    pub host: Addr,
    pub opponent: Option<Addr>,
    pub host_move: GameMove,
    pub opp_move: Option<GameMove>,
    pub result: Option<GameResult>,
}

pub const GAMES: Map<(&Addr, &Addr), GameData> = Map::new("games");

// INDEXED MAP

pub struct GameDataIndexes<'a> {
    // secondary index by opponent address
    // the last key is the primary key which is an auto incremented token counter
    // pub host: MultiIndex<'a, Addr, GameData>,
    pub opponent: MultiIndex<'a, (Addr, Addr), GameData>,
}

// this may become a macro, not important just boilerplate, builds the list of indexes for later use
impl<'a> IndexList<GameData> for GameDataIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<GameData>> + '_> {
        let v: Vec<&dyn Index<GameData>> = vec![&self.opponent];
        Box::new(v.into_iter())
    }
}

pub fn games<'a>() -> IndexedMap<'a, (&'a Addr, &'a Addr), GameData, GameDataIndexes<'a>> {
    let indexes = GameDataIndexes {
        // host: MultiIndex::new(|d| d.host.clone(), "games", "gamedata__host"),
        opponent: MultiIndex::new(|d|
            (d.opponent.clone().unwrap(), d.host.clone()), // opponent needs to be unwrapped, coz it's Option
            "games",
            "gamedata__opponent"),
    };
    IndexedMap::new("games", indexes)
}
