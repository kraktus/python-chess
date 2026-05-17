use shakmaty::{Chess, FromSetup, Position, Role, Square, fen::Fen};
use std::str::FromStr;

fn main() {
    let setup = Fen::from_str("r3k1nr/ppq1pp1p/2p3p1/8/1PPR4/2N5/P3QPPP/5RK1 b kq b3 0 16")
        .unwrap()
        .into_setup();
    let chess = Chess::from_setup(setup, shakmaty::CastlingMode::Standard).unwrap();
    let move_list = chess.legal_moves();
    for m in move_list {
        if m.role() == Role::Queen && m.to() == Square::H2 {
            println!(
                "Move: {:?}, SAN: {}",
                m,
                shakmaty::san::San::from_move(&chess, m)
            );
        }
    }
}
