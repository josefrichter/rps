#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdError, StdResult,
};
use cw2::set_contract_version;

use cw_controllers::{Admin, AdminError, AdminResponse, HookError};
use cw_utils::maybe_addr;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GamesListResponse, InstantiateMsg, QueryMsg};
use crate::state::{games, Game, GameMove, GameResult, ADMIN, BLACKLIST};

const CONTRACT_NAME: &str = "crates.io:rps";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, AdminError> {
    let maybe_admin = maybe_addr(deps.api, Some(msg.admin.to_string()))?;
    ADMIN.set(deps.branch(), maybe_admin)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

type Res<T> = Result<T, Box<dyn std::error::Error>>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> Res<Response> {
    match msg {
        ExecuteMsg::StartGame {
            opponent,
            host_move,
        } => Ok(try_startgame(deps, info, opponent, host_move)?),
        ExecuteMsg::EndGame {
            host,
            opponent_move,
        } => Ok(try_endgame(deps, info, host, opponent_move)?),
        ExecuteMsg::UpdateAdmin { admin } => Ok(try_updateadmin(deps, info, admin)?),
        ExecuteMsg::AddToBlacklist { addr } => Ok(try_addtoblacklist(ADMIN, deps, info, addr)?),
        ExecuteMsg::RemoveFromBlacklist { addr } => {
            Ok(try_removefromblacklist(ADMIN, deps, info, addr)?)
        }
    }
}

pub fn try_startgame(
    deps: DepsMut,
    info: MessageInfo,
    opponent: Addr,
    host_move: GameMove,
) -> Result<Response, ContractError> {
    // check Addr
    let checked_opponent: Addr = deps.api.addr_validate(&opponent.to_string())?;

    // check that not trying to start game against oneself
    if checked_opponent == info.sender {
        return Err(ContractError::GameAgainstYourself {});
    }

    // grab the blacklist
    let blacklist = BLACKLIST.query_hooks(deps.as_ref())?;

    // check if opponent isn't blacklisted
    if blacklist.hooks.contains(&checked_opponent.to_string()) {
        return Err(ContractError::Blacklisted {
            addr: checked_opponent,
        });
    }

    // check if this message sender (i.e. host) isn't blacklisted
    if blacklist.hooks.contains(&info.sender.to_string()) {
        return Err(ContractError::Blacklisted { addr: info.sender });
    }

    // create the Game struct from the submitted data
    let game = Game {
        host: info.sender.clone(),
        opponent: checked_opponent.clone(),
        host_move: host_move,
        opponent_move: None, // first move by host = no move by opponent
        result: None, // result only after opponent move
    };

    // save the new struct and create response accordingly
    match save_game(deps, game) {
        Ok(_game) => Ok(Response::new().add_attribute("method", "try_startgame")),
        Err(e) => Err(e),
    }
}

// helper function to create new game, wrapping IndexedMap update syntax
pub fn save_game(deps: DepsMut, game: Game) -> Result<Game, ContractError> {
    games().update(
        deps.storage,
        generate_key_for_game(&game), // call helper function to get the key
        |old| match old {
            Some(_) => Err(ContractError::DuplicateGame {}), // if the game already exists, update() will fail here. prevents overwriting.
            None => Ok(game), // if it doesn't exist, it's created
        },
    )
}

// helper function to update existing game, wrapping IndexedMap update sytax
// unlike save_game(), this one will fail if the game doesn't exist yet
pub fn update_game(deps: &mut DepsMut, game: Game) -> Result<Game, ContractError> {
    games().update(
        deps.storage,
        generate_key_for_game(&game),
        |old| match old {
            Some(_) => Ok(game), // if game exists, update it
            None => Err(ContractError::GameNotFound {}), // if game doesn't exist yet, fail
        },
    )
}

// helper to wrap IndexedMap delete syntax
pub fn delete_game(deps: &mut DepsMut, game: Game) -> Result<(), StdError> {
    games().remove(deps.storage, generate_key_for_game(&game))
}

// helper that will create (host, opponent) tuple key from Game struct
pub fn generate_key_for_game(game: &Game) -> (Addr, Addr) {
    (game.host.clone(), game.opponent.clone())
}

pub fn try_endgame(
    mut deps: DepsMut,
    info: MessageInfo,
    host: Addr,
    opponent_move: GameMove,
) -> Result<Response, ContractError> {
    // check Addr
    let checked_host: Addr = deps.api.addr_validate(&host.to_string())?;
    // might not be necessary to check host against blacklist, coz he couldn't have started the game if blacklisted
    // however, he could have gotten blacklisted after starting the game...
    let blacklist = BLACKLIST.query_hooks(deps.as_ref())?;

    // check if host is not blacklisted (got blacklisted after the game was started)
    if blacklist.hooks.contains(&checked_host.to_string()) {
        return Err(ContractError::Blacklisted { addr: checked_host });
    }

    // check if this message sender isn't blacklisted
    if blacklist.hooks.contains(&info.sender.to_string()) {
        return Err(ContractError::Blacklisted { addr: info.sender });
    }

    // lookup game by host, opponent
    let opponent = info.sender;
    let game_res = games().may_load(deps.storage, (checked_host, opponent));
    let mut game = match game_res {
        Ok(Some(game)) => game,
        Ok(None) => return Err(ContractError::GameNotFound {}),
        Err(_e) => return Err(ContractError::GameNotFound {}),
    };

    // find out game result
    let result = match (&game.host_move, &opponent_move) {
        // host starts with Rock
        (GameMove::Rock {}, GameMove::Paper {}) => GameResult::OpponentWins {},
        (GameMove::Rock {}, GameMove::Scissors {}) => GameResult::HostWins {},

        // host starts with Paper
        (GameMove::Paper {}, GameMove::Rock {}) => GameResult::HostWins {},
        (GameMove::Paper {}, GameMove::Scissors {}) => GameResult::OpponentWins {},

        // host starts with Scissors
        (GameMove::Scissors {}, GameMove::Rock {}) => GameResult::OpponentWins {},
        (GameMove::Scissors {}, GameMove::Paper {}) => GameResult::HostWins {},

        // same moves = tie
        (move1, move2) if (move1 == move2) => GameResult::Tie {},
        (_, _) => return Err(ContractError::GameResultNotFound {}),
    };

    // update map accordingly
    game.opponent_move = Some(opponent_move);
    game.result = Some(result.clone());

    // ## actually, if I understand the docs here https://academy.terra.money/courses/take/cosmwasm-smart-contracts-i/assignments/27056622-building-out-the-rps-game
    // ## the final state of the game is not saved in cotract
    // ## it's just verifiable on chain (by including it in response?)
    let updated_game = match update_game(&mut deps, game) {
        Ok(game) => game,
        Err(e) => return Err(e),
    };

    // get a human friendly string message
    let result_string = match result {
        GameResult::Tie {} => "Tie",
        GameResult::HostWins {} => "Host won",
        GameResult::OpponentWins {} => "Opponent won",
    };

    // delete the game from the IndexedMap and create a response with game_result
    match delete_game(&mut deps, updated_game) {
        Ok(_) => Ok(Response::new()
            .add_attribute("method", "try_endgame")
            .add_attribute("game_result", result_string)),
        Err(_) => Err(ContractError::CannotFinishGame {}),
    }
}

pub fn try_updateadmin(
    deps: DepsMut,
    info: MessageInfo,
    admin: Addr,
) -> Result<Response, AdminError> {
    // this is using ADMIN controller from cw_controllers
    // to set a new contract admin
    // it also checks that only current admin can set the new admin
    let maybe_admin = maybe_addr(deps.api, Some(admin.to_string()))?;
    ADMIN.execute_update_admin(deps, info, maybe_admin)
}

pub fn try_addtoblacklist(
    admin: Admin,
    deps: DepsMut,
    info: MessageInfo,
    addr: Addr,
) -> Result<Response, HookError> {
    // this is using HOOKS controller from cw_controllers
    let checked_addr = deps.api.addr_validate(&addr.to_string())?;
    BLACKLIST.execute_add_hook(&admin, deps, info, checked_addr)
}

pub fn try_removefromblacklist(
    admin: Admin,
    deps: DepsMut,
    info: MessageInfo,
    addr: Addr,
) -> Result<Response, HookError> {
    let checked_addr = deps.api.addr_validate(&addr.to_string())?;
    BLACKLIST.execute_remove_hook(&admin, deps, info, checked_addr)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetGamesByPlayer { player } => to_binary(&query_games(deps, &player)?),
        QueryMsg::GetGamesByHost { host } => to_binary(&query_games_by_host(deps, &host)?),
        QueryMsg::GetGamesByOpponent { opponent } => {
            to_binary(&query_games_by_opponent(deps, &opponent)?)
        }
        QueryMsg::GetAdmin {} => to_binary(&query_admin(deps)?),
    }
}

fn query_games(deps: Deps, player: &Addr) -> StdResult<GamesListResponse> {
    // get games for given Addr
    // where the Addr is either a host or an opponent
    let mut games = vec![];
    let mut games_by_host = query_games_by_host(deps, player).unwrap().games;
    let mut games_by_opponent = query_games_by_opponent(deps, player).unwrap().games;

    games.append(&mut games_by_host);
    games.append(&mut games_by_opponent);

    Ok(GamesListResponse { games: games })
}

fn query_games_by_host(deps: Deps, host: &Addr) -> StdResult<GamesListResponse> {
    // get games for given host Addr
    let games_by_host = games()
        .idx
        .host
        .prefix(host.clone())
        .range(deps.storage, None, None, Order::Ascending)
        .map(|kv_item| kv_item.unwrap().1) // reformat it so that we get a vector of Game structs
        .collect::<Vec<_>>();

    Ok(GamesListResponse {
        games: games_by_host,
    })
}

fn query_games_by_opponent(deps: Deps, opponent: &Addr) -> StdResult<GamesListResponse> {
    // get games for given opponent Addr
    let games_by_opponent = games()
        .idx
        .opponent
        .prefix(opponent.clone())
        .range(deps.storage, None, None, Order::Ascending)
        .map(|kv_item| kv_item.unwrap().1) // reformat it so that we get a vector of Game structs
        .collect::<Vec<_>>();

    Ok(GamesListResponse {
        games: games_by_opponent,
    })
}

// who's the contract admin?
fn query_admin(deps: Deps) -> StdResult<AdminResponse> {
    Ok(ADMIN.query_admin(deps)?)
}
