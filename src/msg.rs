use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::Addr;
use crate::state::{GameMove, Game};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: Addr,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    // host starts the game by picking opponent and casting first move
    StartGame { opponent: Addr, host_move: GameMove },
    // opponent ends game against host by casting a move
    // (there can be only one (host, opponent) game at a time)
    EndGame { host: Addr, opponent_move: GameMove },
    // change contract admin
    UpdateAdmin { admin: Addr },
    // manage blacklist of addresses that cannot participate in game
    AddToBlacklist { addr: Addr },
    RemoveFromBlacklist { addr: Addr }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetGamesByHost { host: Addr },
    GetGamesByOpponent { opponent: Addr },
    // get all games where player is either host or opponent
    GetGamesByPlayer { player: Addr },
    GetAdmin {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GamesListResponse {
    pub games: Vec<Game>,
}
