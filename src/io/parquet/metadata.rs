use std::collections::{HashMap, HashSet};

use crate::array::metadata::{ArrayMetadata, Edges};
use crate::array::CoordType;
use crate::datatypes::{Dimension, GeoDataType};
use crate::error::{GeoArrowError, Result};

use arrow_schema::Schema;
use parquet::file::metadata::FileMetaData;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Top-level GeoParquet file metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeoParquetMetadata {
    /// The version identifier for the GeoParquet specification.
    pub version: String,

    /// The name of the "primary" geometry column. In cases where a GeoParquet file contains
    /// multiple geometry columns, the primary geometry may be used by default in geospatial
    /// operations.
    pub primary_column: String,

    /// Metadata about geometry columns. Each key is the name of a geometry column in the table.
    pub columns: HashMap<String, GeoParquetColumnMetadata>,
}

/// GeoParquet column metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeoParquetColumnMetadata {
    /// Name of the geometry encoding format. As of GeoParquet 1.1, `"WKB"`, `"point"`,
    /// `"linestring"`, `"polygon"`, `"multipoint"`, `"multilinestring"`, and `"multipolygon"` are
    /// supported.
    pub encoding: String,

    /// The geometry types of all geometries, or an empty array if they are not known.
    ///
    /// This field captures the geometry types of the geometries in the column, when known.
    /// Accepted geometry types are: `"Point"`, `"LineString"`, `"Polygon"`, `"MultiPoint"`,
    /// `"MultiLineString"`, `"MultiPolygon"`, `"GeometryCollection"`.
    ///
    /// In addition, the following rules are used:
    ///
    /// - In case of 3D geometries, a `" Z"` suffix gets added (e.g. `["Point Z"]`).
    /// - A list of multiple values indicates that multiple geometry types are present (e.g.
    ///   `["Polygon", "MultiPolygon"]`).
    /// - An empty array explicitly signals that the geometry types are not known.
    /// - The geometry types in the list must be unique (e.g. `["Point", "Point"]` is not valid).
    ///
    /// It is expected that this field is strictly correct. For example, if having both polygons
    /// and multipolygons, it is not sufficient to specify `["MultiPolygon"]`, but it is expected
    /// to specify `["Polygon", "MultiPolygon"]`. Or if having 3D points, it is not sufficient to
    /// specify `["Point"]`, but it is expected to list `["Point Z"]`.
    pub geometry_types: Vec<String>,

    /// [PROJJSON](https://proj.org/specifications/projjson.html) object representing the
    /// Coordinate Reference System (CRS) of the geometry. If the field is not provided, the
    /// default CRS is [OGC:CRS84](https://www.opengis.net/def/crs/OGC/1.3/CRS84), which means the
    /// data in this column must be stored in longitude, latitude based on the WGS84 datum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crs: Option<Value>,

    /// Winding order of exterior ring of polygons. If present must be `"counterclockwise"`;
    /// interior rings are wound in opposite order. If absent, no assertions are made regarding the
    /// winding order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<String>,

    /// Name of the coordinate system for the edges. Must be one of `"planar"` or `"spherical"`.
    /// The default value is `"planar"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edges: Option<String>,

    /// Bounding Box of the geometries in the file, formatted according to RFC 7946, section 5.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Vec<f64>>,

    /// Coordinate epoch in case of a dynamic CRS, expressed as a decimal year.
    ///
    /// In a dynamic CRS, coordinates of a point on the surface of the Earth may change with time.
    /// To be unambiguous, the coordinates must always be qualified with the epoch at which they
    /// are valid.
    ///
    /// The optional epoch field allows to specify this in case the crs field defines a dynamic
    /// CRS. The coordinate epoch is expressed as a decimal year (e.g. `2021.47`). Currently, this
    /// specification only supports an epoch per column (and not per geometry).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epoch: Option<f64>,

    /// Object containing bounding box column names to help accelerate spatial data retrieval
    #[serde(skip_serializing_if = "Option::is_none")]
    pub covering: Option<HashMap<String, Value>>,
}

impl GeoParquetMetadata {
    /// Construct a [`GeoParquetMetadata`] from Parquet [`FileMetaData`]
    pub fn from_parquet_meta(metadata: &FileMetaData) -> Result<Self> {
        let kv_metadata = metadata.key_value_metadata();

        if let Some(metadata) = kv_metadata {
            for kv in metadata {
                if kv.key == "geo" {
                    if let Some(value) = &kv.value {
                        return Ok(serde_json::from_str(value)?);
                    }
                }
            }
        }

        Err(GeoArrowError::General(
            "expected a 'geo' key in GeoParquet metadata".to_string(),
        ))
    }

    /// Update a GeoParquetMetadata from another file's metadata
    ///
    /// This will expand the bounding box of each geometry column to include the bounding box
    /// defined in the other file's GeoParquet metadata
    pub fn try_update(&mut self, other: &FileMetaData) -> Result<()> {
        let other = Self::from_parquet_meta(other)?;
        self.try_compatible_with(&other)?;
        for (column_name, column_meta) in self.columns.iter_mut() {
            let other_column_meta = other.columns.get(column_name.as_str()).unwrap();
            match (column_meta.bbox.as_mut(), &other_column_meta.bbox) {
                (Some(bbox), Some(other_bbox)) => {
                    assert_eq!(bbox.len(), other_bbox.len());
                    if bbox.len() == 4 {
                        if other_bbox[0] < bbox[0] {
                            bbox[0] = other_bbox[0];
                        }
                        if other_bbox[1] < bbox[1] {
                            bbox[1] = other_bbox[1];
                        }
                        if other_bbox[2] > bbox[2] {
                            bbox[2] = other_bbox[2];
                        }
                        if other_bbox[3] > bbox[3] {
                            bbox[3] = other_bbox[3];
                        }
                    } else if bbox.len() == 6 {
                        if other_bbox[0] < bbox[0] {
                            bbox[0] = other_bbox[0];
                        }
                        if other_bbox[1] < bbox[1] {
                            bbox[1] = other_bbox[1];
                        }
                        if other_bbox[2] < bbox[2] {
                            bbox[2] = other_bbox[2];
                        }
                        if other_bbox[3] > bbox[3] {
                            bbox[3] = other_bbox[3];
                        }
                        if other_bbox[4] > bbox[4] {
                            bbox[4] = other_bbox[4];
                        }
                        if other_bbox[5] > bbox[5] {
                            bbox[5] = other_bbox[5];
                        }
                    }
                }
                (None, Some(other_bbox)) => {
                    column_meta.bbox = Some(other_bbox.clone());
                }
                // If the RHS doesn't have a bbox, we don't need to update
                (_, None) => {}
            }
        }
        Ok(())
    }

    /// Check if this metadata is compatible with another metadata instance, swallowing the error
    /// message if not compatible.
    pub fn is_compatible_with(&self, other: &GeoParquetMetadata) -> bool {
        self.try_compatible_with(other).is_ok()
    }

    /// Assert that this metadata is compatible with another metadata instance, erroring if not
    pub fn try_compatible_with(&self, other: &GeoParquetMetadata) -> Result<()> {
        if self.version.as_str() != other.version.as_str() {
            return Err(GeoArrowError::General(
                "Different GeoParquet versions".to_string(),
            ));
        }

        if self.primary_column.as_str() != other.primary_column.as_str() {
            return Err(GeoArrowError::General(
                "Different GeoParquet primary columns".to_string(),
            ));
        }

        for key in self.columns.keys() {
            let left = self.columns.get(key).unwrap();
            let right = other
                .columns
                .get(key)
                .ok_or(GeoArrowError::General(format!(
                    "Other GeoParquet metadata missing column {}",
                    key
                )))?;

            if left.encoding.as_str() != right.encoding.as_str() {
                return Err(GeoArrowError::General(format!(
                    "Different GeoParquet encodings for column {}",
                    key
                )));
            }

            if left.geometry_types != right.geometry_types {
                return Err(GeoArrowError::General(format!(
                    "Different GeoParquet geometry types for column {}",
                    key
                )));
            }

            if let (Some(left_bbox), Some(right_bbox)) = (&left.bbox, &right.bbox) {
                if left_bbox.len() != right_bbox.len() {
                    return Err(GeoArrowError::General(format!(
                        "Different bbox dimensions for column {}",
                        key
                    )));
                }
            }

            match (left.crs.as_ref(), right.crs.as_ref()) {
                (Some(left_crs), Some(right_crs)) => {
                    if left_crs != right_crs {
                        return Err(GeoArrowError::General(format!(
                            "Different GeoParquet CRS for column {}",
                            key
                        )));
                    }
                }
                (Some(_), None) | (None, Some(_)) => {
                    return Err(GeoArrowError::General(format!(
                        "Different GeoParquet CRS for column {}",
                        key
                    )));
                }
                (None, None) => (),
            }
        }

        Ok(())
    }
}

impl From<GeoParquetColumnMetadata> for ArrayMetadata {
    fn from(value: GeoParquetColumnMetadata) -> Self {
        let edges = if let Some(edges) = value.edges {
            if edges.as_str() == "spherical" {
                Some(Edges::Spherical)
            } else {
                None
            }
        } else {
            None
        };
        ArrayMetadata {
            crs: value.crs,
            edges,
        }
    }
}

impl From<&GeoParquetColumnMetadata> for ArrayMetadata {
    fn from(value: &GeoParquetColumnMetadata) -> Self {
        value.clone().into()
    }
}
// TODO: deduplicate with `resolve_types` in `downcast.rs`
pub(crate) fn infer_geo_data_type(
    geometry_types: &HashSet<&str>,
    coord_type: CoordType,
) -> Result<Option<GeoDataType>> {
    if geometry_types.iter().any(|t| t.contains(" Z")) {
        return Err(GeoArrowError::General(
            "3D coordinates not currently supported".to_string(),
        ));
    }

    match geometry_types.len() {
        0 => Ok(None),
        1 => Ok(Some(match *geometry_types.iter().next().unwrap() {
            "Point" => GeoDataType::Point(coord_type, Dimension::XY),
            "LineString" => GeoDataType::LineString(coord_type, Dimension::XY),
            "Polygon" => GeoDataType::Polygon(coord_type, Dimension::XY),
            "MultiPoint" => GeoDataType::MultiPoint(coord_type, Dimension::XY),
            "MultiLineString" => GeoDataType::MultiLineString(coord_type, Dimension::XY),
            "MultiPolygon" => GeoDataType::MultiPolygon(coord_type, Dimension::XY),
            "GeometryCollection" => GeoDataType::GeometryCollection(coord_type, Dimension::XY),
            _ => unreachable!(),
        })),
        2 => {
            if geometry_types.contains("Point") && geometry_types.contains("MultiPoint") {
                Ok(Some(GeoDataType::MultiPoint(coord_type, Dimension::XY)))
            } else if geometry_types.contains("LineString")
                && geometry_types.contains("MultiLineString")
            {
                Ok(Some(GeoDataType::MultiLineString(
                    coord_type,
                    Dimension::XY,
                )))
            } else if geometry_types.contains("Polygon") && geometry_types.contains("MultiPolygon")
            {
                Ok(Some(GeoDataType::MultiPolygon(coord_type, Dimension::XY)))
            } else {
                Ok(Some(GeoDataType::Mixed(coord_type, Dimension::XY)))
            }
        }
        _ => Ok(Some(GeoDataType::Mixed(coord_type, Dimension::XY))),
    }
}

/// Find all geometry columns in the Arrow schema, constructing their GeoDataTypes
pub(crate) fn find_geoparquet_geom_columns(
    metadata: &FileMetaData,
    schema: &Schema,
    coord_type: CoordType,
) -> Result<Vec<(usize, Option<GeoDataType>)>> {
    let meta = GeoParquetMetadata::from_parquet_meta(metadata)?;

    meta.columns
        .iter()
        .map(|(col_name, col_meta)| {
            let geometry_column_index = schema
                .fields()
                .iter()
                .position(|field| field.name().as_str() == col_name.as_str())
                .unwrap();
            let mut geometry_types = HashSet::with_capacity(col_meta.geometry_types.len());
            col_meta.geometry_types.iter().for_each(|t| {
                geometry_types.insert(t.as_str());
            });
            Ok((
                geometry_column_index,
                infer_geo_data_type(&geometry_types, coord_type)?,
            ))
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    // We want to ensure that extra keys in future GeoParquet versions do not break
    // By default, serde allows and ignores unknown keys
    #[test]
    fn extra_keys_in_column_metadata() {
        let s = r#"{
            "encoding": "wkb",
            "geometry_types": ["point"],
            "other_key": true
        }"#;
        let meta: GeoParquetColumnMetadata = serde_json::from_str(s).unwrap();
        assert_eq!(meta.encoding, "wkb");
        assert_eq!(meta.geometry_types, vec!["point"]);

        dbg!(&meta);
    }
}
