from __future__ import annotations

from typing import Protocol, Self, Tuple

import numpy as np
from numpy.typing import NDArray

class _ArrowArrayExportable(Protocol):
    """An Arrow or GeoArrow array from a local or remote (e.g. geoarrow.c) source."""
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...

class PointArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...
    def area(self) -> Float64Array: ...
    def bounding_rect(self) -> RectArray: ...
    def center(self) -> PointArray: ...
    def centroid(self) -> PointArray: ...
    def chamberlain_duquette_signed_area(self) -> Float64Array: ...
    def chamberlain_duquette_unsigned_area(self) -> Float64Array: ...
    def convex_hull(self) -> PolygonArray: ...
    def geodesic_area_signed(self) -> Float64Array: ...
    def geodesic_area_unsigned(self) -> Float64Array: ...
    def geodesic_perimeter(self) -> Float64Array: ...
    def is_empty(self) -> BooleanArray: ...
    def length(self) -> Float64Array: ...
    @classmethod
    def from_wkb(cls, input: _ArrowArrayExportable) -> Self: ...
    def to_wkb(self) -> WKBArray: ...

class LineStringArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...
    def area(self) -> Float64Array: ...
    def bounding_rect(self) -> RectArray: ...
    def center(self) -> PointArray: ...
    def centroid(self) -> PointArray: ...
    def chaikin_smoothing(self, n_iterations: int) -> Self: ...
    def chamberlain_duquette_signed_area(self) -> Float64Array: ...
    def chamberlain_duquette_unsigned_area(self) -> Float64Array: ...
    def convex_hull(self) -> PolygonArray: ...
    def densify(self) -> Self: ...
    def geodesic_area_signed(self) -> Float64Array: ...
    def geodesic_area_unsigned(self) -> Float64Array: ...
    def geodesic_perimeter(self) -> Float64Array: ...
    def is_empty(self) -> BooleanArray: ...
    def length(self) -> Float64Array: ...
    @classmethod
    def from_wkb(cls, input: _ArrowArrayExportable) -> Self: ...
    def to_wkb(self) -> WKBArray: ...

class PolygonArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...
    def area(self) -> Float64Array: ...
    def bounding_rect(self) -> RectArray: ...
    def center(self) -> PointArray: ...
    def centroid(self) -> PointArray: ...
    def chaikin_smoothing(self, n_iterations: int) -> Self: ...
    def chamberlain_duquette_signed_area(self) -> Float64Array: ...
    def chamberlain_duquette_unsigned_area(self) -> Float64Array: ...
    def convex_hull(self) -> PolygonArray: ...
    def densify(self) -> Self: ...
    def geodesic_area_signed(self) -> Float64Array: ...
    def geodesic_area_unsigned(self) -> Float64Array: ...
    def geodesic_perimeter(self) -> Float64Array: ...
    def is_empty(self) -> BooleanArray: ...
    @classmethod
    def from_wkb(cls, input: _ArrowArrayExportable) -> Self: ...
    def to_wkb(self) -> WKBArray: ...

class MultiPointArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...
    def area(self) -> Float64Array: ...
    def bounding_rect(self) -> RectArray: ...
    def center(self) -> PointArray: ...
    def centroid(self) -> PointArray: ...
    def chamberlain_duquette_signed_area(self) -> Float64Array: ...
    def chamberlain_duquette_unsigned_area(self) -> Float64Array: ...
    def convex_hull(self) -> PolygonArray: ...
    def geodesic_area_signed(self) -> Float64Array: ...
    def geodesic_area_unsigned(self) -> Float64Array: ...
    def geodesic_perimeter(self) -> Float64Array: ...
    def is_empty(self) -> BooleanArray: ...
    def length(self) -> Float64Array: ...
    @classmethod
    def from_wkb(cls, input: _ArrowArrayExportable) -> Self: ...
    def to_wkb(self) -> WKBArray: ...

class MultiLineStringArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...
    def area(self) -> Float64Array: ...
    def bounding_rect(self) -> RectArray: ...
    def center(self) -> PointArray: ...
    def centroid(self) -> PointArray: ...
    def chaikin_smoothing(self, n_iterations: int) -> Self: ...
    def chamberlain_duquette_signed_area(self) -> Float64Array: ...
    def chamberlain_duquette_unsigned_area(self) -> Float64Array: ...
    def convex_hull(self) -> PolygonArray: ...
    def densify(self) -> Self: ...
    def geodesic_area_signed(self) -> Float64Array: ...
    def geodesic_area_unsigned(self) -> Float64Array: ...
    def geodesic_perimeter(self) -> Float64Array: ...
    def is_empty(self) -> BooleanArray: ...
    def length(self) -> Float64Array: ...
    @classmethod
    def from_wkb(cls, input: _ArrowArrayExportable) -> Self: ...
    def to_wkb(self) -> WKBArray: ...

class MultiPolygonArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...
    def area(self) -> Float64Array: ...
    def bounding_rect(self) -> RectArray: ...
    def center(self) -> PointArray: ...
    def centroid(self) -> PointArray: ...
    def chaikin_smoothing(self, n_iterations: int) -> Self: ...
    def chamberlain_duquette_signed_area(self) -> Float64Array: ...
    def chamberlain_duquette_unsigned_area(self) -> Float64Array: ...
    def convex_hull(self) -> PolygonArray: ...
    def densify(self) -> Self: ...
    def geodesic_area_signed(self) -> Float64Array: ...
    def geodesic_area_unsigned(self) -> Float64Array: ...
    def geodesic_perimeter(self) -> Float64Array: ...
    def is_empty(self) -> BooleanArray: ...
    @classmethod
    def from_wkb(cls, input: _ArrowArrayExportable) -> Self: ...
    def to_wkb(self) -> WKBArray: ...

class MixedGeometryArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...
    def area(self) -> Float64Array: ...
    def bounding_rect(self) -> RectArray: ...
    def center(self) -> PointArray: ...
    def centroid(self) -> PointArray: ...
    def chamberlain_duquette_signed_area(self) -> Float64Array: ...
    def chamberlain_duquette_unsigned_area(self) -> Float64Array: ...
    def convex_hull(self) -> PolygonArray: ...
    def geodesic_area_signed(self) -> Float64Array: ...
    def geodesic_area_unsigned(self) -> Float64Array: ...
    def geodesic_perimeter(self) -> Float64Array: ...
    def is_empty(self) -> BooleanArray: ...
    @classmethod
    def from_ewkb(cls, input: _ArrowArrayExportable) -> Self: ...
    @classmethod
    def from_wkb(cls, input: _ArrowArrayExportable) -> Self: ...
    @classmethod
    def from_wkt(cls, input: _ArrowArrayExportable) -> Self: ...
    def to_wkb(self) -> WKBArray: ...

class GeometryCollectionArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...
    def area(self) -> Float64Array: ...
    def bounding_rect(self) -> RectArray: ...
    def center(self) -> PointArray: ...
    def centroid(self) -> PointArray: ...
    def chamberlain_duquette_signed_area(self) -> Float64Array: ...
    def chamberlain_duquette_unsigned_area(self) -> Float64Array: ...
    def convex_hull(self) -> PolygonArray: ...
    def geodesic_area_signed(self) -> Float64Array: ...
    def geodesic_area_unsigned(self) -> Float64Array: ...
    def geodesic_perimeter(self) -> Float64Array: ...
    def is_empty(self) -> BooleanArray: ...
    @classmethod
    def from_ewkb(cls, input: _ArrowArrayExportable) -> Self: ...
    @classmethod
    def from_wkb(cls, input: _ArrowArrayExportable) -> Self: ...
    @classmethod
    def from_wkt(cls, input: _ArrowArrayExportable) -> Self: ...
    def to_wkb(self) -> WKBArray: ...

class WKBArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...

class RectArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...

class BooleanArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...

class Float16Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...

class Float32Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...
    def to_numpy(self) -> NDArray[np.float32]: ...

class Float64Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...
    def to_numpy(self) -> NDArray[np.float64]: ...

class Int16Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...
    def to_numpy(self) -> NDArray[np.int16]: ...

class Int32Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...
    def to_numpy(self) -> NDArray[np.int32]: ...

class Int64Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...
    def to_numpy(self) -> NDArray[np.int64]: ...

class Int8Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...
    def to_numpy(self) -> NDArray[np.int8]: ...

class LargeStringArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...

class StringArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...

class UInt16Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...
    def to_numpy(self) -> NDArray[np.uint16]: ...

class UInt32Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...
    def to_numpy(self) -> NDArray[np.uint32]: ...

class UInt64Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...
    def to_numpy(self) -> NDArray[np.uint64]: ...

class UInt8Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def __arrow_c_array__(
        self, requested_schema: object | None = None
    ) -> Tuple[object, object]: ...
    def to_numpy(self) -> NDArray[np.uint8]: ...

class ChunkedPointArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def area(self) -> ChunkedFloat64Array: ...
    def bounding_rect(self) -> ChunkedRectArray: ...
    def center(self) -> ChunkedPointArray: ...
    def centroid(self) -> ChunkedPointArray: ...
    def chamberlain_duquette_signed_area(self) -> ChunkedFloat64Array: ...
    def chamberlain_duquette_unsigned_area(self) -> ChunkedFloat64Array: ...
    def concatenate(self) -> PointArray: ...
    def convex_hull(self) -> ChunkedPolygonArray: ...
    def geodesic_area_signed(self) -> ChunkedFloat64Array: ...
    def geodesic_area_unsigned(self) -> ChunkedFloat64Array: ...
    def geodesic_perimeter(self) -> ChunkedFloat64Array: ...
    def is_empty(self) -> BooleanArray: ...
    def length(self) -> ChunkedFloat64Array: ...

class ChunkedLineStringArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def area(self) -> ChunkedFloat64Array: ...
    def bounding_rect(self) -> ChunkedRectArray: ...
    def center(self) -> ChunkedPointArray: ...
    def centroid(self) -> ChunkedPointArray: ...
    def chaikin_smoothing(self, n_iterations: int) -> Self: ...
    def chamberlain_duquette_signed_area(self) -> ChunkedFloat64Array: ...
    def chamberlain_duquette_unsigned_area(self) -> ChunkedFloat64Array: ...
    def concatenate(self) -> LineStringArray: ...
    def convex_hull(self) -> ChunkedPolygonArray: ...
    def densify(self) -> Self: ...
    def geodesic_area_signed(self) -> ChunkedFloat64Array: ...
    def geodesic_area_unsigned(self) -> ChunkedFloat64Array: ...
    def geodesic_perimeter(self) -> ChunkedFloat64Array: ...
    def is_empty(self) -> BooleanArray: ...
    def length(self) -> ChunkedFloat64Array: ...

class ChunkedPolygonArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def area(self) -> ChunkedFloat64Array: ...
    def bounding_rect(self) -> ChunkedRectArray: ...
    def center(self) -> ChunkedPointArray: ...
    def centroid(self) -> ChunkedPointArray: ...
    def chaikin_smoothing(self, n_iterations: int) -> Self: ...
    def chamberlain_duquette_signed_area(self) -> ChunkedFloat64Array: ...
    def chamberlain_duquette_unsigned_area(self) -> ChunkedFloat64Array: ...
    def concatenate(self) -> PolygonArray: ...
    def convex_hull(self) -> ChunkedPolygonArray: ...
    def densify(self) -> Self: ...
    def geodesic_area_signed(self) -> ChunkedFloat64Array: ...
    def geodesic_area_unsigned(self) -> ChunkedFloat64Array: ...
    def geodesic_perimeter(self) -> ChunkedFloat64Array: ...
    def is_empty(self) -> BooleanArray: ...

class ChunkedMultiPointArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def area(self) -> ChunkedFloat64Array: ...
    def bounding_rect(self) -> ChunkedRectArray: ...
    def center(self) -> ChunkedPointArray: ...
    def centroid(self) -> ChunkedPointArray: ...
    def chamberlain_duquette_signed_area(self) -> ChunkedFloat64Array: ...
    def chamberlain_duquette_unsigned_area(self) -> ChunkedFloat64Array: ...
    def concatenate(self) -> MultiPointArray: ...
    def convex_hull(self) -> ChunkedPolygonArray: ...
    def geodesic_area_signed(self) -> ChunkedFloat64Array: ...
    def geodesic_area_unsigned(self) -> ChunkedFloat64Array: ...
    def geodesic_perimeter(self) -> ChunkedFloat64Array: ...
    def is_empty(self) -> BooleanArray: ...
    def length(self) -> ChunkedFloat64Array: ...

class ChunkedMultiLineStringArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def area(self) -> ChunkedFloat64Array: ...
    def bounding_rect(self) -> ChunkedRectArray: ...
    def center(self) -> ChunkedPointArray: ...
    def centroid(self) -> ChunkedPointArray: ...
    def chaikin_smoothing(self, n_iterations: int) -> Self: ...
    def chamberlain_duquette_signed_area(self) -> ChunkedFloat64Array: ...
    def chamberlain_duquette_unsigned_area(self) -> ChunkedFloat64Array: ...
    def concatenate(self) -> MultiLineStringArray: ...
    def convex_hull(self) -> ChunkedPolygonArray: ...
    def densify(self) -> Self: ...
    def geodesic_area_signed(self) -> ChunkedFloat64Array: ...
    def geodesic_area_unsigned(self) -> ChunkedFloat64Array: ...
    def geodesic_perimeter(self) -> ChunkedFloat64Array: ...
    def is_empty(self) -> BooleanArray: ...
    def length(self) -> ChunkedFloat64Array: ...

class ChunkedMultiPolygonArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def area(self) -> ChunkedFloat64Array: ...
    def bounding_rect(self) -> ChunkedRectArray: ...
    def center(self) -> ChunkedPointArray: ...
    def centroid(self) -> ChunkedPointArray: ...
    def chaikin_smoothing(self, n_iterations: int) -> Self: ...
    def chamberlain_duquette_signed_area(self) -> ChunkedFloat64Array: ...
    def chamberlain_duquette_unsigned_area(self) -> ChunkedFloat64Array: ...
    def concatenate(self) -> MultiPolygonArray: ...
    def convex_hull(self) -> ChunkedPolygonArray: ...
    def densify(self) -> Self: ...
    def geodesic_area_signed(self) -> ChunkedFloat64Array: ...
    def geodesic_area_unsigned(self) -> ChunkedFloat64Array: ...
    def geodesic_perimeter(self) -> ChunkedFloat64Array: ...
    def is_empty(self) -> BooleanArray: ...

class ChunkedMixedGeometryArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def area(self) -> ChunkedFloat64Array: ...
    def bounding_rect(self) -> ChunkedRectArray: ...
    def center(self) -> ChunkedPointArray: ...
    def centroid(self) -> ChunkedPointArray: ...
    def chamberlain_duquette_signed_area(self) -> ChunkedFloat64Array: ...
    def chamberlain_duquette_unsigned_area(self) -> ChunkedFloat64Array: ...
    def concatenate(self) -> MixedGeometryArray: ...
    def convex_hull(self) -> ChunkedPolygonArray: ...
    def geodesic_area_signed(self) -> ChunkedFloat64Array: ...
    def geodesic_area_unsigned(self) -> ChunkedFloat64Array: ...
    def geodesic_perimeter(self) -> ChunkedFloat64Array: ...
    def is_empty(self) -> BooleanArray: ...

class ChunkedGeometryCollectionArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def area(self) -> ChunkedFloat64Array: ...
    def bounding_rect(self) -> ChunkedRectArray: ...
    def center(self) -> ChunkedPointArray: ...
    def centroid(self) -> ChunkedPointArray: ...
    def chamberlain_duquette_signed_area(self) -> ChunkedFloat64Array: ...
    def chamberlain_duquette_unsigned_area(self) -> ChunkedFloat64Array: ...
    def concatenate(self) -> GeometryCollectionArray: ...
    def convex_hull(self) -> ChunkedPolygonArray: ...
    def geodesic_area_signed(self) -> ChunkedFloat64Array: ...
    def geodesic_area_unsigned(self) -> ChunkedFloat64Array: ...
    def geodesic_perimeter(self) -> ChunkedFloat64Array: ...
    def is_empty(self) -> BooleanArray: ...

class ChunkedWKBArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...

class ChunkedRectArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...

class ChunkedBooleanArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...

class ChunkedFloat16Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...

class ChunkedFloat32Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...

class ChunkedFloat64Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...

class ChunkedInt16Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...

class ChunkedInt32Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...

class ChunkedInt64Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...

class ChunkedInt8Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...

class ChunkedLargeStringArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...

class ChunkedStringArray:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...

class ChunkedUInt16Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...

class ChunkedUInt32Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...

class ChunkedUInt64Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...

class ChunkedUInt8Array:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...

class GeoTable:
    def __eq__(self, other: Self) -> bool: ...
    def __len__(self) -> int: ...
    def tmp(self) -> None:
        """test"""

# Operations
def area(input: _ArrowArrayExportable) -> Float64Array: ...
def signed_area(input: _ArrowArrayExportable) -> Float64Array: ...
def center(input: _ArrowArrayExportable) -> PointArray: ...
def centroid(input: _ArrowArrayExportable) -> PointArray: ...
def convex_hull(input: _ArrowArrayExportable) -> PolygonArray: ...

# I/O
def from_ewkb(
    input: _ArrowArrayExportable,
) -> (
    PointArray
    | LineStringArray
    | PolygonArray
    | MultiPointArray
    | MultiLineStringArray
    | MultiPolygonArray
    | MixedGeometryArray
    | GeometryCollectionArray
): ...
def from_wkb(
    input: _ArrowArrayExportable,
) -> (
    PointArray
    | LineStringArray
    | PolygonArray
    | MultiPointArray
    | MultiLineStringArray
    | MultiPolygonArray
    | MixedGeometryArray
    | GeometryCollectionArray
): ...
def from_wkt(
    input: _ArrowArrayExportable,
) -> (
    PointArray
    | LineStringArray
    | PolygonArray
    | MultiPointArray
    | MultiLineStringArray
    | MultiPolygonArray
    | MixedGeometryArray
    | GeometryCollectionArray
): ...
def to_wkb(input: _ArrowArrayExportable) -> WKBArray: ...
def read_csv(path: str, geometry_column_name: str) -> GeoTable: ...
def read_flatgeobuf(path: str) -> GeoTable: ...
def read_geojson(path: str) -> GeoTable: ...
