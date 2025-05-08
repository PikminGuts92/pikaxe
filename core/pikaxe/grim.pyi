class ArkOffsetEntry:
    id: int
    path: str
    offset: int
    part: int
    size: int
    inflated_size: int

class Ark:
    version: int
    encryption: int | None
    entries: list[ArkOffsetEntry]

    @staticmethod
    def from_file_path(path: str) -> Ark: ...

class Bitmap:
    bpp: int
    encoding: int
    mip_maps: int
    width: int
    height: int
    bpl: int
    raw_data: list[int]

    @staticmethod
    def from_file_path(path: str) -> Bitmap: ...

    def save_to_file(path: str) -> None: ...