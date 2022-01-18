use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

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
    NotCastYet {},
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
pub enum GameResult {
    HostWins {},
    OpponentWins {},
    Tie {},
    NotDecidedYet {},
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema)]
pub struct GameData {
    pub host: Addr,
    pub opponent: Addr,
    pub host_move: GameMove,
    pub opp_move: GameMove,
    pub result: GameResult,
}

pub const GAMES: Map<(&Addr, &Addr), GameData> = Map::new("games");
