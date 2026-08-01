import chess

import pyrust_chess
pyrust_chess.patch_supported(
    dst_module=chess,
    src_module=pyrust_chess,
)

board = chess.BaseBoard(chess.STARTING_BOARD_FEN)


try:
    board.pieces_mask(99, chess.WHITE)
except AssertionError:
    pass