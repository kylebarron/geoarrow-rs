//! This is partially derived from https://github.com/alttch/myval under the Apache 2 license

use arrow_schema::{DataType, Field, SchemaBuilder};
use geozero::wkb::Ewkb;
use geozero::{ColumnValue, FeatureProcessor, GeomProcessor, GeozeroGeometry, PropertyProcessor};
// use chrono::{DateTime, NaiveDateTime, Utc};
use futures::stream::TryStreamExt;
use sqlx::postgres::PgRow;
use sqlx::{Column, Executor, Postgres, Row, TypeInfo};
use std::sync::Arc;

use crate::error::Result;
use crate::io::geozero::array::mixed::MixedGeometryStreamBuilder;
use crate::io::geozero::table::{GeoTableBuilder, GeoTableBuilderOptions};
use crate::table::GeoTable;
use crate::trait_::GeometryArrayBuilder;

// TODO: right now this uses a hashmap with names. In the future, it should switch to using a
// positional schema.
// TODO: manage buffering
impl<G: GeometryArrayBuilder + GeomProcessor> GeoTableBuilder<G> {
    fn add_postgres_geometry(&mut self, value: &[u8]) -> Result<()> {
        self.geometry_begin()?;
        let ewkb = Ewkb(value.to_vec());
        ewkb.process_geom(self)?;
        self.geometry_end()?;
        Ok(())
    }

    fn add_postgres_row(&mut self, row_idx: u64, row: &PgRow) -> Result<()> {
        self.feature_begin(row_idx)?;
        self.properties_begin()?;
        let mut geometry: Option<&[u8]> = None;
        for (i, column) in row.columns().iter().enumerate() {
            match column.name() {
                "geometry" => {
                    geometry = Some(row.try_get(i)?);
                }
                column_name => {
                    let column_value: ColumnValue = match column.type_info().name() {
                        "BOOL" => ColumnValue::Bool(row.try_get(i)?),
                        "INT2" => ColumnValue::Short(row.try_get(i)?),
                        "INT4" => ColumnValue::Int(row.try_get(i)?),
                        "INT8" => ColumnValue::Long(row.try_get(i)?),
                        // // "TIMESTAMP" => DataType::Timestamp(<_>::default()),
                        // // "TIMESTAMPTZ" => Data::TimestampTz(<_>::default()),
                        "FLOAT4" => ColumnValue::Float(row.try_get(i)?),
                        "FLOAT8" => ColumnValue::Double(row.try_get(i)?),
                        "VARCHAR" | "CHAR" => ColumnValue::String(row.try_get(i)?),
                        "JSON" | "JSONB" => ColumnValue::String(row.try_get(i)?),
                        v => todo!("unimplemented type in column value: {}", v),
                    };
                    self.property(i, column_name, &column_value)?;
                }
            }
        }
        self.properties_end()?;
        // Add geometry after we've finished writing properties
        self.add_postgres_geometry(geometry.expect("missing geometry for row {}"))?;
        self.feature_end(row_idx)?;
        Ok(())
    }

    fn initialize_from_row(row: &PgRow, mut options: GeoTableBuilderOptions) -> Result<Self> {
        let mut schema = SchemaBuilder::new();
        for column in row.columns() {
            let column_name = column.name();
            // hack
            if column_name == "geometry" {
                continue;
            }
            let data_type = match column.type_info().name() {
                "BOOL" => DataType::Boolean,
                "INT2" => DataType::Int16,
                "INT4" => DataType::Int32,
                "INT8" => DataType::Int64,
                // "TIMESTAMP" => DataType::Timestamp(<_>::default()),
                // "TIMESTAMPTZ" => Data::TimestampTz(<_>::default()),
                "FLOAT4" => DataType::Float32,
                "FLOAT8" => DataType::Float64,
                "VARCHAR" | "CHAR" => DataType::Utf8,
                "JSON" | "JSONB" => DataType::Utf8,
                v => todo!("unimplemented type: {}", v),
            };

            schema.push(Field::new(column_name, data_type, true))
        }
        options.properties_schema = Some(Arc::new(schema.finish()));

        // Create builder and add this row
        let mut builder = Self::new_with_options(options);
        builder.add_postgres_row(0, row)?;
        Ok(builder)
    }
}

pub async fn read_postgis<'c, E: Executor<'c, Database = Postgres>>(
    executor: E,
    sql: &str,
) -> Result<Option<GeoTable>> {
    let query = sqlx::query::<Postgres>(sql);
    let mut result_stream = query.fetch(executor);

    let mut table_builder: Option<GeoTableBuilder<MixedGeometryStreamBuilder<i32>>> = None;
    let mut row_idx = 0;
    while let Some(row) = result_stream.try_next().await? {
        if let Some(ref mut table_builder) = table_builder {
            // Add this row
            table_builder.add_postgres_row(row_idx, &row)?;
        } else {
            // Initialize table builder
            let table_builder_options = GeoTableBuilderOptions::default();
            table_builder = Some(GeoTableBuilder::initialize_from_row(
                &row,
                table_builder_options,
            )?)
        };
        row_idx += 1;
    }

    if let Some(table_builder) = table_builder {
        Ok(Some(table_builder.finish()?))
    } else {
        Ok(None)
    }
}
