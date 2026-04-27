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
    _patch_from_module(
        dst_module=dst_module,
        src_module=src_module,
        names=["SquareSet", "Piece"],
    )