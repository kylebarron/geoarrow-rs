from __future__ import annotations

from typing import Protocol, Tuple, Literal

AreaMethodT = Literal["ellipsoidal", "euclidean", "spherical"]
"""Acceptable strings to be passed into the `method` parameter for
[`area`][geoarrow.rust.core.area] and
[`signed_area`][geoarrow.rust.core.signed_area].
"""

class ArrowArrayExportable(Protocol):
    """An Arrow or GeoArrow array from an Arrow producer (e.g. geoarrow.c or pyarrow)."""

    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]:
        ...


class ArrowStreamExportable(Protocol):
    """An Arrow or GeoArrow ChunkedArray or Table from an Arrow producer (e.g. geoarrow.c or pyarrow)."""

    def __arrow_c_stream__(self, requested_schema: object | None = None) -> object:
        ...
