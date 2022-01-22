Rock-Paper-Scissors
-------------------

Smart contract game. Terra blockchain, Rust language.

This is an exercise based on Terra Academy videos https://academy.terra.money/

One player starts a game by choosing an opponent Addr and castin a vote Rock/Paper/Scissors:

```rust
StartGame { opponent: Addr, host_move: GameMove }
```

The whole game state is kept in this struct:

```rust
pub struct Game {
    pub host: Addr,
    pub opponent: Addr,
    pub host_move: GameMove,
    // optional, not known at the start of the game
    pub opponent_move: Option<GameMove>,
    // optional, not known before opponent_move is known
    pub result: Option<GameResult>,
}
```

Then the opponent can cast their move in the same game:

```rust
EndGame { host: Addr, opponent_move: GameMove }
```

The Games are stored in an IndexedMap (https://docs.rs/cw-storage-plus/0.11.1/cw_storage_plus/struct.IndexedMap.html),
where unique primary key for each Game us a tuple `(host, opponent)`. That means, there is always only one game between host and opponent (but opponent can initiate separate game against host). It is possible to add one more `id` to the primary key tuple, so that it looks something like `(host, opponent, id)`. This way it would be possible to have multiple games between any two opponents at any time.

- The `EndGame` will look up the game knowing the host and opponent,
- will evaluate who won (Rock vs Paper vs Scissors),
- will *delete* the game from the `IndexedMap`,
- and will return a response with info who won.

The code is using *admin* controller from `cw_controllers` to store/retrieve/update contract admin. It is also using *hooks* controller from `cw_controllers` to hold a list of blacklisted addresses. Blacklisted addresses cannot start or participate in games.

This is built using cosmwasm `1.0.0-beta` version, while Terra blockchain still runs on `0.16.2` now.

I hear that especially `IndexedMap` wouldn't work on `0.16.2`, but haven't tried myself yet. If that's the case, it is possible to replace `IndexedMap` with standard `Map`, and build the additional indexes (Games by host, Games by opponent, Games by `(host,opponent)` tuple) as separate maps. I.e. the main `Map` holds sometthing like `(game_id, Game)`, and the other maps are something like `(host_addr, game_id)`, listing all game ids for given host, and `(opponent_addr, game_id)` listing all game ids for given opponent.

However these indices are used only for query functions, so that they can be used in UI to show games for given Addr. The core game logic doesn't need them - when the opponent vote is being cast, the `(host, opponent)` tuple is known at that point.

There is very limited documentation for `IndexedMap` right now (Januery 2022), I used mainly these resources:

- https://github.com/CosmWasm/cw-plus/blob/main/packages/storage-plus/README.md
- https://docs.cosmwasm.com/tutorials/storage/indexes (this one helped a lot)
- https://github.com/oraichain/oraiwasm/blob/master/package/plus/market_offering_storage/src/state.rs (these guys' implementation helped me figure out various tiny details)
- https://docs.rs/cw-storage-plus/0.11.1/cw_storage_plus/struct.IndexedMap.html (of course)

I couldn't find much more, and couldn't find solutions to Terra Academy tasks, so hopefully making this public will help someone who ran into the same problem. Feel free to contact me - see github profile, or ping me on Terra discord.
