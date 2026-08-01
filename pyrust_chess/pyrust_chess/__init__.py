from .pyrust_chess import *


def _patch_from_module(dst_module, src_module, names):
    """
    Replace attributes on dst_module with attributes from src_module for each name.
    """
    for name in names:
        try:
            setattr(dst_module, name, getattr(src_module, name))
        except Exception as e:
            print(f"Couldn't monkey-patch [{name}], err: {e}")


def _patch_status(src_module, dst_module):
    """
    Replace the STATUS_* module-level constants on dst_module with the ones
    defined on the pyrust_chess.Status class.
    """
    status_cls = getattr(src_module, "Status")
    names = [name for name in dir(dst_module) if name.startswith("STATUS_")]
    _patch_from_module(dst_module=dst_module, src_module=status_cls, names=names)


# cannot reference itself as module
def _patch_supported(src_module, dst_module):
    _patch_from_module(
        dst_module=dst_module,
        src_module=src_module,
        # DO NOT MONKEY-PATCH Board and BaseBoard
        names=["SquareSet", "Piece", "Move", "Termination", "Outcome", "Status", "InvalidMoveError", "AmbiguousMoveError", "IllegalMoveError"],
    )
    _patch_status(src_module=src_module, dst_module=dst_module)

def patch_chess():
    """
    Monkey-patch the chess module classes with the pyrust_chess implementations.

    The following classes and constants will be replaced in the chess module:
    - SquareSet
    - Piece
    - Move
    - Termination
    - Outcome
    - Status
    
    As well as the errors:
    - InvalidMoveError
    - AmbiguousMoveError
    - IllegalMoveError
    """
    try:
        import chess
    except ImportError:
        # warn
        import warnings
        warnings.warn("Could not import chess module, skipping monkey-patching. Consider removing `pyrust_chess.patch_chess` if you not import the `chess`/`python-chess` module.")

    _patch_supported(src_module=pyrust_chess, dst_module=chess)






