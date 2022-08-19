use enum_iterator::{all, All};
use four_player_chess_async::chess_clock::ChessClock;
use four_player_chess_async::ingame::Ingame;
use four_player_chess_async::Ident::First;
use four_player_chess_async::{GameBootstrap, Ident};
use std::iter::Cycle;

pub(crate) struct GameFabric {
    bootstrap: Option<GameBootstrap>,
    next_player: Cycle<All<Ident>>,
}

impl GameFabric {
    pub(crate) fn new() -> GameFabric {
        GameFabric {
            bootstrap: None,
            next_player: all::<Ident>().cycle(),
        }
    }

    pub(crate) fn join(&mut self) -> Ingame {
        let p = self.next_player.next().unwrap();
        if p == First {
            self.bootstrap = Some(GameBootstrap::new(ChessClock::default()));
        }
        self.bootstrap.as_mut().unwrap().join(p).unwrap()
    }
}
