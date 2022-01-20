use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use cw_controllers::{Admin};

pub const ADMIN: Admin = Admin::new("admin");

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
