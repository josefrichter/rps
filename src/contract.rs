// use std::ops::RangeBounds;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;

use cw_controllers::{Admin, AdminError, AdminResponse, HookError};
use cw_utils::maybe_addr;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GamesListResponse, InstantiateMsg, QueryMsg};
use crate::state::games;
use crate::state::{GameData, GameMove, ADMIN, BLACKLIST, GAMES};

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
    let blacklist = BLACKLIST.query_hooks(deps.as_ref())?;

    if blacklist.hooks.contains(&checked_opponent.to_string()) {
        return Err(ContractError::Blacklisted {
            addr: checked_opponent,
        });
    }

    if blacklist.hooks.contains(&info.sender.to_string()) {
        return Err(ContractError::Blacklisted { addr: info.sender });
    }

    let store = deps.storage;
    let gamedata = GameData {
        host: info.sender.clone(),                // TODO: need to clone() here?
        opponent: Some(checked_opponent.clone()), // TODO: need to clone() here?
        host_move: host_move,
        opp_move: None,
        result: None,
    };

    GAMES.save(store, (&info.sender, &checked_opponent), &gamedata)?;
    Ok(Response::new().add_attribute("method", "try_startgame"))
}

pub fn try_updateadmin(
    deps: DepsMut,
    info: MessageInfo,
    admin: Addr,
) -> Result<Response, AdminError> {
    let maybe_admin = maybe_addr(deps.api, Some(admin.to_string()))?;
    ADMIN.execute_update_admin(deps, info, maybe_admin)
}

pub fn try_addtoblacklist(
    admin: Admin,
    deps: DepsMut,
    info: MessageInfo,
    addr: Addr,
) -> Result<Response, HookError> {
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
        QueryMsg::GetGamesByPlayer { player } => to_binary(&query_games(deps, player)?),
        QueryMsg::GetGamesByHost { host } => to_binary(&query_games_by_host(deps, host)?),
        QueryMsg::GetGamesByOpponent { opponent } => {
            to_binary(&query_games_by_opponent(deps, opponent)?)
        }
        QueryMsg::GetAdmin {} => to_binary(&query_admin(deps)?),
    }
}

fn query_games(deps: Deps, player: Addr) -> StdResult<GamesListResponse> {
    let games_by_player = GAMES
        .range(deps.storage, None, None, Order::Ascending)
        .flat_map(|r| match r {
            Ok((_, data))
                if data.host == Addr::unchecked(&player)
                    || data.opponent == Some(Addr::unchecked(&player)) =>
            {
                Some(data)
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    Ok(GamesListResponse {
        games: games_by_player,
    })
}

fn query_games_by_host(deps: Deps, host: Addr) -> StdResult<GamesListResponse> {
    let games_by_host = GAMES
        .prefix(&host)
        .range(deps.storage, None, None, Order::Ascending)
        .flat_map(|item| match item {
            Ok((_, data)) => Some(data),
            _ => None,
        })
        .collect::<Vec<_>>();

    Ok(GamesListResponse {
        games: games_by_host,
    })
}

fn query_games_by_opponent(deps: Deps, opponent: Addr) -> StdResult<GamesListResponse> {
    let games_by_opponent = GAMES
        .range(deps.storage, None, None, Order::Ascending)
        .flat_map(|r| match r {
            Ok((_, data)) if data.opponent == Some(Addr::unchecked(&opponent)) => Some(data),
            _ => None,
        })
        .collect::<Vec<_>>();

    Ok(GamesListResponse {
        games: games_by_opponent,
    })
}

fn query_admin(deps: Deps) -> StdResult<AdminResponse> {
    Ok(ADMIN.query_admin(deps)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::GamesListResponse;
    use crate::state::{GameData, GameMove};
    use cosmwasm_std::testing::{
        mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info,
    };
    use cosmwasm_std::{coins, from_binary};
    // use cw_storage_plus::I64Key;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg {
            admin: Addr::unchecked("bobby"),
        };

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn start_game() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            opponent: Addr::unchecked(""),
            host_move: GameMove::Scissors {},
        };
        let _err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        // assert!(
        //     matches!(err, ContractError::Std(StdError::GenericErr { msg: _ })),
        // );

        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            opponent: Addr::unchecked("oprah"),
            host_move: GameMove::Scissors {},
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0); // TODO: is this a correct test?
    }

    #[test]
    fn add_to_and_remove_from_blacklist() {
        let mut deps = mock_dependencies();

        // instantiate by "creator", with admin "bobby"
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg {
            admin: Addr::unchecked("bobby"),
        };
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // add "black" to blacklist
        let info = mock_info("bobby", &coins(2, "token"));
        let msg = ExecuteMsg::AddToBlacklist {
            addr: Addr::unchecked("black"),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0); // TODO: is this a correct test?

        // check "black" is on blacklist
        let blacklist = BLACKLIST.query_hooks(deps.as_ref()).unwrap();
        assert_eq!(blacklist.hooks, ["black"]);

        // remove "black" from blacklist
        let info = mock_info("bobby", &coins(2, "token"));
        let msg = ExecuteMsg::RemoveFromBlacklist {
            addr: Addr::unchecked("black"),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0); // TODO: is this a correct test?

        // check blacklist is empty now
        let blacklist = BLACKLIST.query_hooks(deps.as_ref()).unwrap();
        assert_eq!(blacklist.hooks, [] as [&str; 0]);
    }

    #[test]
    fn blacklisted_addr_cannot_start_game() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        // instantiate by "creator", with admin "creator"
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg {
            admin: Addr::unchecked("creator"),
        };
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // add "black" to blacklist
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::AddToBlacklist {
            addr: Addr::unchecked("black"),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0); // TODO: is this a correct test?

        // start game by "black" against "anyone" should fail
        let info = mock_info("black", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            opponent: Addr::unchecked("anyone"),
            host_move: GameMove::Scissors {},
        };
        let err = execute(deps.as_mut(), mock_env(), info, msg);
        let err_unwrapped = err.unwrap_err().downcast::<ContractError>().unwrap();
        assert_eq!(
            *err_unwrapped,
            ContractError::Blacklisted {
                addr: Addr::unchecked("black")
            }
        );

        // remove "black" from blacklist
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::RemoveFromBlacklist {
            addr: Addr::unchecked("black"),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        // start game by "black" agains "anyone" again should succeed now
        let info = mock_info("black", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            opponent: Addr::unchecked("anyone"),
            host_move: GameMove::Scissors {},
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0);
    }

    #[test]
    fn cannot_start_game_against_blacklisted() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        // instantiate by "creator", with admin "creator"
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg {
            admin: Addr::unchecked("creator"),
        };
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // add "black" to blacklist
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::AddToBlacklist {
            addr: Addr::unchecked("black"),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0); // TODO: is this a correct test?

        // start game by "creator" against "black" should fail
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            opponent: Addr::unchecked("black"),
            host_move: GameMove::Scissors {},
        };
        let err = execute(deps.as_mut(), mock_env(), info, msg);
        let err_unwrapped = err.unwrap_err().downcast::<ContractError>().unwrap();
        assert_eq!(
            *err_unwrapped,
            ContractError::Blacklisted {
                addr: Addr::unchecked("black")
            }
        );

        // remove "black" from blacklist
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::RemoveFromBlacklist {
            addr: Addr::unchecked("black"),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        // start game by "creator" agains "black" again should succeed now
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            opponent: Addr::unchecked("black"),
            host_move: GameMove::Scissors {},
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0);
    }

    #[test]
    fn update_admin() {
        // instantiate by "creator", setting "bobby" as admin
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg {
            admin: Addr::unchecked("bobby"),
        };

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // now only "bobby" can update admin
        let info = mock_info("bobby", &coins(2, "token"));
        let msg = ExecuteMsg::UpdateAdmin {
            admin: Addr::unchecked("adrianne"),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0); // TODO: is this a correct test?
    }

    #[test]
    fn query_games() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        // games for "tony" should be empty initially
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetGamesByPlayer {
                player: Addr::unchecked("tony"),
            },
        )
        .unwrap();
        let gameslist: GamesListResponse = from_binary(&res).unwrap();
        assert_eq!(gameslist.games, []);

        // create game by "jimmy" against "oprah"
        let info = mock_info("jimmy", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            opponent: Addr::unchecked("oprah"),
            host_move: GameMove::Scissors {},
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // create game by "tony" against "oprah"
        let info = mock_info("tony", &coins(2, "token"));
        let msg = ExecuteMsg::StartGame {
            opponent: Addr::unchecked("oprah"),
            host_move: GameMove::Scissors {},
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // check that "tony"'s game is returned
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetGamesByPlayer {
                player: Addr::unchecked("tony"),
            },
        )
        .unwrap();
        let tonys_gameslist: GamesListResponse = from_binary(&res).unwrap();
        assert_eq!(
            tonys_gameslist.games,
            [GameData {
                host: Addr::unchecked("tony"),
                opponent: Some(Addr::unchecked("oprah")),
                host_move: GameMove::Scissors {},
                opp_move: None,
                result: None
            }]
        )
    }

    #[test]
    fn query_admin() {
        // instantiate by "creator", setting "bobby" as admin
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg {
            admin: Addr::unchecked("bobby"),
        };

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // check that admin is "bobby"
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAdmin {}).unwrap();
        let adminresponse: AdminResponse = from_binary(&res).unwrap();
        assert_eq!(adminresponse.admin, Some("bobby".to_string()));
    }

    #[test]
    fn gamedata_indexedmap() {
        // let mut store = MockStorage::new();
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let host1 = Addr::unchecked("host1");
        let host2 = Addr::unchecked("host2");
        let opponent1 = Addr::unchecked("opponent1");
        let opponent2 = Addr::unchecked("opponent2");

        // host1, opponent1
        let game11 = GameData {
            host: host1.clone(),
            opponent: Some(opponent1.clone()),
            host_move: GameMove::Rock {},
            opp_move: None,
            result: None,
        };

        // host1, opponent2
        let game12 = GameData {
            host: host1.clone(),
            opponent: Some(opponent2.clone()),
            host_move: GameMove::Paper {},
            opp_move: None,
            result: None,
        };

        // host2, opponent2
        let game22 = GameData {
            host: host2.clone(),
            opponent: Some(opponent2.clone()),
            host_move: GameMove::Paper {},
            opp_move: None,
            result: None,
        };

        games()
            .update(
                &mut deps.storage,
                (&game11.clone().host, &game11.clone().opponent.unwrap()),
                |old| match old {
                    Some(_) => Err(ContractError::DuplicateGame {}),
                    None => Ok(game11.clone()),
                },
            )
            .unwrap();

        games()
            .update(
                &mut deps.storage,
                (&game12.clone().host, &game12.clone().opponent.unwrap()),
                |old| match old {
                    Some(_) => Err(ContractError::DuplicateGame {}),
                    None => Ok(game12.clone()),
                },
            )
            .unwrap();

        games()
            .update(
                &mut deps.storage,
                (&game22.clone().host, &game22.clone().opponent.unwrap()),
                |old| match old {
                    Some(_) => Err(ContractError::DuplicateGame {}),
                    None => Ok(game22.clone()),
                },
            )
            .unwrap();


        // load all games
        let list = games()
            .range(&mut deps.storage, None, None, Order::Ascending)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        println!("=== all games ===");
        println!("{:?}", list);
        let (_, t) = &list[0];
        assert_eq!(t, &game11);
        assert_eq!(3, list.len());


        // load games for host1
        let list = games()
            .idx
            .host
            .prefix(host1.clone())
            .range(&mut deps.storage, None, None, Order::Ascending)
            .collect::<Result<Vec<(_,_)>, _>>()
            .unwrap();

        println!("=== games for host1 ===");
        println!("{:?}", list);
        let (_, t) = &list[0];
        assert_eq!(t, &game11);
        assert_eq!(2, list.len());


        // load games for opponent2
        let list = games()
            .idx
            .opponent
            .prefix(opponent2.clone())
            .range(&mut deps.storage, None, None, Order::Ascending)
            .collect::<Result<Vec<(_,_)>, _>>()
            .unwrap();

        println!("=== games for opponent2 ===");
        println!("{:?}", list);
        let (_, t) = &list[0];
        assert_eq!(t, &game12);
        assert_eq!(2, list.len());


        // let keys: Vec<_> = games()
        //     .keys_raw(&mut deps.storage, None, None, Order::Ascending)
        //     .collect();
        // println!("{:?}", keys);
    }
}
