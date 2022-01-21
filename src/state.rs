use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex, UniqueIndex};

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
    // TODO: decide which approach is needed here
    // for host, the pkey is specified as (Addr,Addr) tuple
    // so that the lookup can return vector like [((Addr,Addr),Gamedata)]
    pub host: MultiIndex<'a, Addr, GameData, (Addr, Addr)>,
    // without specifying pkey type, the returned vector is like [((), GameData)]
    // which in next step can be mapped to just [GameData, GameData,...] - is the key needed at any point?
    pub opponent: MultiIndex<'a, Addr, GameData>,
    pub host_opponent_id: UniqueIndex<'a, (Addr, Addr), GameData>,
}

// this may become a macro, not important just boilerplate, builds the list of indexes for later use
impl<'a> IndexList<GameData> for GameDataIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<GameData>> + '_> {
        let v: Vec<&dyn Index<GameData>> = vec![&self.host, &self.opponent, &self.host_opponent_id];
        Box::new(v.into_iter())
    }
}

pub fn games<'a>() -> IndexedMap<'a, (Addr, Addr), GameData, GameDataIndexes<'a>> {
    let indexes = GameDataIndexes {
        host: MultiIndex::new(|d|
            d.host.clone(), // opponent needs to be unwrapped, coz it's Option
            "games",
            "gamedata__host"),
        opponent: MultiIndex::new(|d|
            d.opponent.clone().unwrap(), // opponent needs to be unwrapped, coz it's Option
            "games",
            "gamedata__opponent"),
        host_opponent_id: UniqueIndex::new(|d|
            (d.host.clone(), d.opponent.clone().unwrap()),
            "gamedata__host_opponent_id")
    };
    IndexedMap::new("games", indexes)
}
