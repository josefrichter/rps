use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::Addr;
use crate::state::{GameMove, GameData};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: Addr,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    StartGame { opponent: Addr, host_move: GameMove },
    UpdateAdmin { admin: Addr },
    AddToBlacklist { addr: Addr },
    RemoveFromBlacklist { addr: Addr }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetGamesByPlayer { player: Addr },
    GetGamesByHost { host: Addr },
    GetGamesByOpponent { opponent: Addr },
    GetAdmin {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GamesListResponse {
    pub games: Vec<GameData>,
}
