from .rust_chess import *

def _patch_from_module(dst_module, src_module, names):
    """
    Replace attributes on dst_module with attributes from src_module for each name.
    """
    for name in names:
        try:
            setattr(dst_module, name, getattr(src_module, name))
        except Exception as e:
            print(f"Couldn't monkey-patch [{name}], err: {e}")


# cannot reference itself as module
def patch_supported(src_module, dst_module):
    patch_board_init(dst_module)
    patch_baseboard(dst_module, src_module)
    _patch_from_module(
        dst_module=dst_module,
        src_module=src_module,
        names=["SquareSet", "Piece", "Move"],
    )

def patch_board_init(dst_module):
    # Patch chess.Board.__init__ to use super() instead of BaseBoard.__init__(self, ...)
    original_init = dst_module.Board.__init__
    def new_init(self, fen=dst_module.STARTING_FEN, chess960=False):
        super(dst_module.Board, self).__init__(None)
        self.chess960 = chess960
        self.ep_square = None
        self.move_stack = []
        self._stack = []
        if fen is None:
            self.clear()
        elif fen == dst_module.STARTING_FEN:
            self.reset()
        else:
            self.set_fen(fen)
    dst_module.Board.__init__ = new_init

def patch_baseboard(dst_module, src_module):
    # We will copy missing methods from dst_module.BaseBoard to src_module.BaseBoard
    # Then replace dst_module.BaseBoard with src_module.BaseBoard
    dst_bb = dst_module.BaseBoard
    src_bb = src_module.BaseBoard
    
    for name in dir(dst_bb):
        if not name.startswith("__") and not hasattr(src_bb, name):
            setattr(src_bb, name, getattr(dst_bb, name))
            
    # Explicitly copy __eq__ and others if not in src_bb
    # (they are already in src_bb natively, so skip)
    
    dst_module.BaseBoard = src_bb
