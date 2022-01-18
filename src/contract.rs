#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, Order};
use cw2::set_contract_version;

use crate::error::{ContractError};
use crate::msg::{GamesListResponse, ExecuteMsg, QueryMsg};
use crate::state::{State, STATE, GameData, GAMES, GameResult, GameMove};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:rps";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::StartGame { opponent, host_move } => try_startgame(deps, info, opponent, host_move),
    }
}

pub fn try_startgame(deps: DepsMut, info: MessageInfo, opponent: Addr, host_move: GameMove) -> Result<Response, ContractError> {
    // check Addr
    let checked_opponent: Addr = deps.api.addr_validate(&opponent.to_string())?;

    let store = deps.storage;
    let gamedata = GameData {
        host: info.sender.clone(), // TODO: need to clone() here?
        opponent:  checked_opponent.clone(), // TODO: need to clone() here?
        host_move: host_move,
        opp_move: GameMove::NotCastYet {},
        result: GameResult::NotDecidedYet {},
    };

    GAMES.save(store, (&info.sender, &checked_opponent), &gamedata)?;
    Ok(Response::new().add_attribute("method", "try_startgame"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetGamesByPlayer { player } => to_binary(&query_games(deps, player)?),
        QueryMsg::GetGamesByHost { host } => to_binary(&query_games_by_host(deps, host)?),
        QueryMsg::GetGamesByOpponent { opponent } => to_binary(&query_games_by_opponent(deps, opponent)?),
    }
}

fn query_games(deps: Deps, player: Addr) -> StdResult<GamesListResponse> {

    let games_by_player = GAMES
        .range(deps.storage, None, None, Order::Ascending)
        .flat_map(|r| match r {
            Ok((_, data)) if data.host == Addr::unchecked(&player) || data.opponent == Addr::unchecked(&player) => Some(data),
            _ => None
        })
        .collect::<Vec<_>>();


    Ok(GamesListResponse { games: games_by_player })
}

fn query_games_by_host(deps: Deps, host: Addr) -> StdResult<GamesListResponse> {
    let games_by_host = GAMES
        .prefix(&host)
        .range(deps.storage, None, None, Order::Ascending)
        .flat_map(|item| match item {
                    Ok((_, data)) => Some(data),
                    _ => None
                })
        .collect::<Vec<_>>();

    Ok(GamesListResponse { games: games_by_host })
}

fn query_games_by_opponent(deps: Deps, opponent: Addr) -> StdResult<GamesListResponse> {

    let games_by_opponent = GAMES
        .range(deps.storage, None, None, Order::Ascending)
        .flat_map(|r| match r {
            Ok((_, data)) if data.opponent == Addr::unchecked(&opponent) => Some(data),
            _ => None
        })
        .collect::<Vec<_>>();

    Ok(GamesListResponse { games: games_by_opponent })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, StdError};
    use crate::msg::{GamesListResponse};
    use crate::state::{GameData, GameMove, GameResult};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn start_game() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame { opponent: Addr::unchecked(""), host_move: GameMove::Scissors {}};
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert!(
            matches!(err, ContractError::Std(StdError::GenericErr { msg: _ })),
        );

        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame { opponent: Addr::unchecked("oprah"), host_move: GameMove::Scissors {}};
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0); // TODO: is this a correct test?
    }

    #[test]
    fn query_games() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        // games for "tony" should be empty initially
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGamesByPlayer{ player: Addr::unchecked("tony") }).unwrap();
        let gameslist: GamesListResponse = from_binary(&res).unwrap();
        assert_eq!(gameslist.games, []);

        // create game by "jimmy" against "oprah"
        let info = mock_info("jimmy", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame { opponent: Addr::unchecked("oprah"), host_move: GameMove::Scissors {}};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // create game by "tony" against "oprah"
        let info = mock_info("tony", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame { opponent: Addr::unchecked("oprah"), host_move: GameMove::Scissors {}};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // check that "tony"'s game is returned
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGamesByPlayer{ player: Addr::unchecked("tony") }).unwrap();
        let tonys_gameslist: GamesListResponse = from_binary(&res).unwrap();
        assert_eq!(
            tonys_gameslist.games,
            [GameData { host: Addr::unchecked("tony"), opponent: Addr::unchecked("oprah"), host_move: GameMove::Scissors {}, opp_move: GameMove::NotCastYet {}, result: GameResult::NotDecidedYet {} }]
        )
    }
}
