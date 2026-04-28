import chess

import rust_chess
rust_chess.patch_supported(
    dst_module=chess,
    src_module=rust_chess,
)

board = chess.BaseBoard(chess.STARTING_BOARD_FEN)


try:
    board.pieces_mask(99, chess.WHITE)
except AssertionError:
    pass