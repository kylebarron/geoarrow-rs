use geo::{
    CoordNum, Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon,
    Point, Polygon, Rect,
};

use super::{
    GeometryCollectionTrait, LineStringTrait, MultiLineStringTrait, MultiPointTrait,
    MultiPolygonTrait, PointTrait, PolygonTrait, RectTrait,
};

#[allow(clippy::type_complexity)]
pub trait GeometryTrait {
    type T: CoordNum;
    type Point<'a>: 'a + PointTrait<T = Self::T>
    where
        Self: 'a;
    type LineString<'a>: 'a + LineStringTrait<T = Self::T>
    where
        Self: 'a;
    type Polygon<'a>: 'a + PolygonTrait<T = Self::T>
    where
        Self: 'a;
    type MultiPoint<'a>: 'a + MultiPointTrait<T = Self::T>
    where
        Self: 'a;
    type MultiLineString<'a>: 'a + MultiLineStringTrait<T = Self::T>
    where
        Self: 'a;
    type MultiPolygon<'a>: 'a + MultiPolygonTrait<T = Self::T>
    where
        Self: 'a;
    type GeometryCollection<'a>: 'a + GeometryCollectionTrait<T = Self::T>
    where
        Self: 'a;
    type Rect<'a>: 'a + RectTrait<T = Self::T>
    where
        Self: 'a;

    fn as_type(
        &self,
    ) -> GeometryType<
        '_,
        Self::Point<'_>,
        Self::LineString<'_>,
        Self::Polygon<'_>,
        Self::MultiPoint<'_>,
        Self::MultiLineString<'_>,
        Self::MultiPolygon<'_>,
        Self::GeometryCollection<'_>,
        Self::Rect<'_>,
    >;
}

#[derive(Debug)]
pub enum GeometryType<'a, P, L, Y, MP, ML, MY, GC, R>
where
    P: PointTrait,
    L: LineStringTrait,
    Y: PolygonTrait,
    MP: MultiPointTrait,
    ML: MultiLineStringTrait,
    MY: MultiPolygonTrait,
    GC: GeometryCollectionTrait,
    R: RectTrait,
{
    Point(&'a P),
    LineString(&'a L),
    Polygon(&'a Y),
    MultiPoint(&'a MP),
    MultiLineString(&'a ML),
    MultiPolygon(&'a MY),
    GeometryCollection(&'a GC),
    Rect(&'a R),
}

impl<'a, T: CoordNum + 'a> GeometryTrait for Geometry<T> {
    type T = T;
    type Point = Point<Self::T>;
    type LineString = LineString<Self::T>;
    type Polygon = Polygon<Self::T>;
    type MultiPoint = MultiPoint<Self::T>;
    type MultiLineString = MultiLineString<Self::T>;
    type MultiPolygon = MultiPolygon<Self::T>;
    type GeometryCollection = GeometryCollection<Self::T>;
    type Rect = Rect<Self::T>;

    fn as_type(
        &'a self,
    ) -> GeometryType<
        'a,
        Point<T>,
        LineString<T>,
        Polygon<T>,
        MultiPoint<T>,
        MultiLineString<T>,
        MultiPolygon<T>,
        GeometryCollection<T>,
        Rect<T>,
    > {
        match self {
            Geometry::Point(p) => GeometryType::Point(p),
            Geometry::LineString(p) => GeometryType::LineString(p),
            Geometry::Polygon(p) => GeometryType::Polygon(p),
            Geometry::MultiPoint(p) => GeometryType::MultiPoint(p),
            Geometry::MultiLineString(p) => GeometryType::MultiLineString(p),
            Geometry::MultiPolygon(p) => GeometryType::MultiPolygon(p),
            Geometry::GeometryCollection(p) => GeometryType::GeometryCollection(p),
            Geometry::Rect(p) => GeometryType::Rect(p),
            _ => todo!(),
        }
    }
}
