use std::fmt;

use arrow_array::OffsetSizeTrait;
use geo::MapCoordsInPlace;
use geozero::ToWkt;

use crate::io::geo::{
    geometry_collection_to_geo, geometry_to_geo, line_string_to_geo, multi_line_string_to_geo,
    multi_point_to_geo, multi_polygon_to_geo, point_to_geo, polygon_to_geo, rect_to_geo,
};
use crate::scalar::*;

/// Write geometry to display formatter
/// This takes inspiration from Shapely, which prints a max of 80 characters for the geometry:
/// https://github.com/shapely/shapely/blob/c3ddf310f108a7f589d763d613d755ac12ab5d4f/shapely/geometry/base.py#L163-L177
fn write_geometry(f: &mut fmt::Formatter<'_>, mut geom: geo::Geometry) -> fmt::Result {
    geom.map_coords_in_place(|geo::Coord { x, y }| geo::Coord {
        x: (x * 1000.0).round() / 1000.0,
        y: (y * 1000.0).round() / 1000.0,
    });

    let wkt = geom.to_wkt().unwrap();

    // the total length is limited to 80 characters including brackets
    let max_length = 78;
    write!(f, "<")?;
    if wkt.len() > max_length {
        let trimmed_wkt = wkt.chars().take(max_length - 3).collect::<String>();
        f.write_str(trimmed_wkt.as_str())?;
        write!(f, "...")?;
    } else {
        f.write_str(wkt.as_str())?;
    }
    write!(f, ">")?;
    Ok(())
}

impl fmt::Display for Point<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let geo_geometry = geo::Geometry::Point(point_to_geo(self));
        write_geometry(f, geo_geometry)
    }
}

impl fmt::Display for Rect<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let geo_geometry = geo::Geometry::Rect(rect_to_geo(self));
        write_geometry(f, geo_geometry)
    }
}

macro_rules! impl_fmt {
    ($struct_name:ty, $conversion_fn:ident, $geo_geom_type:path) => {
        impl<O: OffsetSizeTrait> fmt::Display for $struct_name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let geo_geometry = $geo_geom_type($conversion_fn(self));
                write_geometry(f, geo_geometry)
            }
        }
    };
}

impl_fmt!(
    LineString<'_, O>,
    line_string_to_geo,
    geo::Geometry::LineString
);
impl_fmt!(Polygon<'_, O>, polygon_to_geo, geo::Geometry::Polygon);
impl_fmt!(
    MultiPoint<'_, O>,
    multi_point_to_geo,
    geo::Geometry::MultiPoint
);
impl_fmt!(
    MultiLineString<'_, O>,
    multi_line_string_to_geo,
    geo::Geometry::MultiLineString
);
impl_fmt!(
    MultiPolygon<'_, O>,
    multi_polygon_to_geo,
    geo::Geometry::MultiPolygon
);
impl_fmt!(
    GeometryCollection<'_, O>,
    geometry_collection_to_geo,
    geo::Geometry::GeometryCollection
);

impl<O: OffsetSizeTrait> fmt::Display for Geometry<'_, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let geo_geometry = geometry_to_geo(self);
        write_geometry(f, geo_geometry)
    }
}

#[cfg(test)]
mod test {
    use crate::array::PointArray;
    use crate::test::{multipolygon, point};
    use crate::trait_::GeometryArrayAccessor;

    #[test]
    fn test_display_point() {
        let point_array = point::point_array();
        let result = point_array.value(0).to_string();
        let expected = "<POINT(0 1)>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_display_point_5_decimals() {
        let point = geo::Point::from((0.12345, 1.23456));
        let point_array: PointArray = vec![point].as_slice().into();
        let result = point_array.value(0).to_string();
        let expected = "<POINT(0.123 1.235)>";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_display_multipolygon() {
        let multipolygon_array = multipolygon::mp_array();
        let result = multipolygon_array.value(0).to_string();
        let expected =
            "<MULTIPOLYGON(((-111 45,-111 41,-104 41,-104 45,-111 45)),((-111 45,-111 41,...>";
        assert_eq!(result, expected);
    }
}
