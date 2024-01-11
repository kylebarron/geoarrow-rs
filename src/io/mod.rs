//! Contains reader and writer implementations of many common geospatial file formats, including
//! interoperability with the `geozero` crate.

#[cfg(feature = "csv")]
pub mod csv;
#[cfg(feature = "flatgeobuf")]
pub mod flatgeobuf;
#[cfg(feature = "geozero")]
pub mod geojson;
#[cfg(feature = "geozero")]
pub mod geojson_lines;
#[cfg(feature = "geos")]
pub(crate) mod geos;
#[cfg(feature = "geozero")]
pub mod geozero;
#[cfg(feature = "parquet")]
pub mod parquet;
#[cfg(feature = "postgis")]
pub mod postgis;
pub mod wkb;
