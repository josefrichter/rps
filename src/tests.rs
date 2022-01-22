use crate::contract::*;

use cw_controllers::AdminResponse;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GamesListResponse, InstantiateMsg, QueryMsg};
use crate::state::{games, Game, GameMove, BLACKLIST};

use cosmwasm_std::testing::{
    mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info,
};
use cosmwasm_std::{coins, from_binary, Addr, Order};

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

    // starting game against invalid Addr will fail
    let info = mock_info("anyone", &coins(2, "token"));
    let msg = ExecuteMsg::StartGame {
        opponent: Addr::unchecked(""),
        host_move: GameMove::Scissors {},
    };
    let _err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    // assert!(
    //     matches!(err, ContractError::Std(StdError::GenericErr { msg: _ })),
    // );

    // starting game against oneself will fail
    let info = mock_info("oneself", &coins(2, "token"));
    let msg = ExecuteMsg::StartGame {
        opponent: Addr::unchecked("oneself"),
        host_move: GameMove::Scissors {},
    };
    let _err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();

    // starting game with correct data will succeed
    let info = mock_info("anyone", &coins(2, "token"));
    let msg = ExecuteMsg::StartGame {
        opponent: Addr::unchecked("oprah"),
        host_move: GameMove::Scissors {},
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0); // TODO: is this a correct test?
}

#[test]
fn end_game_with_losing_move() {
    // check that sender (opponent) is not equal to host
    // cast opponent vote
    // compare host vs opponent vote to select result
    // return updated Game

    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    // start game with a move
    let info = mock_info("host", &coins(2, "token"));
    let msg = ExecuteMsg::StartGame {
        opponent: Addr::unchecked("opponent"),
        host_move: GameMove::Scissors {},
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    // end game with losing move should succeed
    let info = mock_info("opponent", &coins(2, "token"));
    let msg = ExecuteMsg::EndGame {
        host: Addr::unchecked("host"),
        opponent_move: GameMove::Paper {},
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(res.attributes[1].value, "Host won");
}

#[test]
fn end_game_with_winning_move() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    // start game with a move
    let info = mock_info("host", &coins(2, "token"));
    let msg = ExecuteMsg::StartGame {
        opponent: Addr::unchecked("opponent"),
        host_move: GameMove::Scissors {},
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    // end game with winning move should succeed
    let info = mock_info("opponent", &coins(2, "token"));
    let msg = ExecuteMsg::EndGame {
        host: Addr::unchecked("host"),
        opponent_move: GameMove::Rock {},
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(res.attributes[1].value, "Opponent won");
}

#[test]
fn end_game_with_tie_move() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    // start game with a move
    let info = mock_info("host", &coins(2, "token"));
    let msg = ExecuteMsg::StartGame {
        opponent: Addr::unchecked("opponent"),
        host_move: GameMove::Scissors {},
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    // end game with winning move should succeed
    let info = mock_info("opponent", &coins(2, "token"));
    let msg = ExecuteMsg::EndGame {
        host: Addr::unchecked("host"),
        opponent_move: GameMove::Scissors {},
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);
    assert_eq!(res.attributes[1].value, "Tie");
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
fn blacklisted_cannot_end_game() {
    // this is a situation where a game was started against non-blacklisted opponent
    // and then the opponent got blacklisted
    // so now he's not allowed to end the game (TODO: delete games with blacklisted players?)
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    // instantiate by "creator", with admin "creator"
    let info = mock_info("creator", &coins(1000, "earth"));
    let msg = InstantiateMsg {
        admin: Addr::unchecked("creator"),
    };
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // start game by "creator" against "black" again should succeed initially
    let info = mock_info("creator", &coins(2, "token"));
    let msg = ExecuteMsg::StartGame {
        opponent: Addr::unchecked("black"),
        host_move: GameMove::Scissors {},
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    // add "black" to blacklist
    let info = mock_info("creator", &coins(2, "token"));
    let msg = ExecuteMsg::AddToBlacklist {
        addr: Addr::unchecked("black"),
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0); // TODO: is this a correct test?

    // end game by "black" should fail
    let info = mock_info("black", &coins(2, "token"));
    let msg = ExecuteMsg::EndGame {
        host: Addr::unchecked("creator"),
        opponent_move: GameMove::Scissors {},
    };
    let err = execute(deps.as_mut(), mock_env(), info, msg);
    let err_unwrapped = err.unwrap_err().downcast::<ContractError>().unwrap();
    assert_eq!(
        *err_unwrapped,
        ContractError::Blacklisted {
            addr: Addr::unchecked("black")
        }
    );
}

#[test]
fn cannot_end_game_against_blacklisted() {
    // user starts a game
    // user gets blacklisted
    // opponent tries to end that game
    // it should fail, because host is blacklisted
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    // instantiate by "creator", with admin "creator"
    let info = mock_info("creator", &coins(1000, "earth"));
    let msg = InstantiateMsg {
        admin: Addr::unchecked("admin"),
    };
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // start game by "creator" against "opponent" again should succeed initially
    let info = mock_info("creator", &coins(2, "token"));
    let msg = ExecuteMsg::StartGame {
        opponent: Addr::unchecked("opponent"),
        host_move: GameMove::Scissors {},
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    // add "creator" to blacklist by "admin"
    let info = mock_info("admin", &coins(2, "token"));
    let msg = ExecuteMsg::AddToBlacklist {
        addr: Addr::unchecked("creator"),
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0); // TODO: is this a correct test?

    // end game by "opponent" should fail, because "creator" is blacklisted
    let info = mock_info("opponent", &coins(2, "token"));
    let msg = ExecuteMsg::EndGame {
        host: Addr::unchecked("creator"),
        opponent_move: GameMove::Scissors {},
    };
    let err = execute(deps.as_mut(), mock_env(), info, msg);
    let err_unwrapped = err.unwrap_err().downcast::<ContractError>().unwrap();
    assert_eq!(
        *err_unwrapped,
        ContractError::Blacklisted {
            addr: Addr::unchecked("creator")
        }
    );
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
        [Game {
            host: Addr::unchecked("tony"),
            opponent: Addr::unchecked("oprah"),
            host_move: GameMove::Scissors {},
            opponent_move: None,
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
fn game_indexedmap() {
    // let mut store = MockStorage::new();
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    let host1 = Addr::unchecked("host1");
    let host2 = Addr::unchecked("host2");
    let opponent1 = Addr::unchecked("opponent1");
    let opponent2 = Addr::unchecked("opponent2");

    // host1, opponent1
    let game11 = Game {
        host: host1.clone(),
        opponent: opponent1.clone(),
        host_move: GameMove::Rock {},
        opponent_move: None,
        result: None,
    };

    // host1, opponent2
    let game12 = Game {
        host: host1.clone(),
        opponent: opponent2.clone(),
        host_move: GameMove::Paper {},
        opponent_move: None,
        result: None,
    };

    // host2, opponent2
    let game22 = Game {
        host: host2.clone(),
        opponent: opponent2.clone(),
        host_move: GameMove::Paper {},
        opponent_move: None,
        result: None,
    };

    games()
        .update(
            &mut deps.storage,
            generate_key_for_game(&game11),
            |old| match old {
                Some(_) => Err(ContractError::DuplicateGame {}),
                None => Ok(game11.clone()),
            },
        )
        .unwrap();

    games()
        .update(
            &mut deps.storage,
            generate_key_for_game(&game12),
            |old| match old {
                Some(_) => Err(ContractError::DuplicateGame {}),
                None => Ok(game12.clone()),
            },
        )
        .unwrap();

    games()
        .update(
            &mut deps.storage,
            generate_key_for_game(&game22),
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
        .collect::<Result<Vec<(_, _)>, _>>()
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
        .map(|kv_item| kv_item.unwrap().1)
        .collect::<Vec<_>>();

    println!("=== games for opponent2 ===");
    println!("{:?}", list);
    assert_eq!(list[0], game12);
    assert_eq!(2, list.len());
}
