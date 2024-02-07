use crate::array::CoordType;
use crate::error::Result;
use crate::io::parquet::geoparquet_metadata::build_arrow_schema;
use crate::io::parquet::reader::GeoParquetReaderOptions;
use crate::table::GeoTable;

use futures::stream::TryStreamExt;
use parquet::arrow::arrow_reader::{ArrowReaderMetadata, ArrowReaderOptions};
use parquet::arrow::async_reader::{AsyncFileReader, ParquetRecordBatchStreamBuilder};

/// Asynchronously read a GeoParquet file to a GeoTable.
pub async fn read_geoparquet_async<R: AsyncFileReader + Unpin + Send + 'static>(
    reader: R,
    options: GeoParquetReaderOptions,
) -> Result<GeoTable> {
    let builder = ParquetRecordBatchStreamBuilder::new(reader)
        .await?
        .with_batch_size(options.batch_size);
    read_builder(builder, &options.coord_type).await
}

async fn read_builder<R: AsyncFileReader + Unpin + Send + 'static>(
    builder: ParquetRecordBatchStreamBuilder<R>,
    coord_type: &CoordType,
) -> Result<GeoTable> {
    let (arrow_schema, geometry_column_index, target_geo_data_type) =
        build_arrow_schema(&builder, coord_type)?;

    let stream = builder.build()?;
    let batches = stream.try_collect::<_>().await?;

    GeoTable::from_arrow(
        batches,
        arrow_schema,
        Some(geometry_column_index),
        target_geo_data_type,
    )
}

/// To create from an object-store item:
///
/// ```notest
/// let reader = ParquetObjectReader::new(store, meta);
///
/// ```
pub struct ParquetFile<R: AsyncFileReader + Clone + Unpin + Send + 'static> {
    reader: R,
    meta: ArrowReaderMetadata,
}

impl<R: AsyncFileReader + Clone + Unpin + Send + 'static> ParquetFile<R> {
    pub async fn new(mut reader: R) -> Result<Self> {
        let options = ArrowReaderOptions::new().with_page_index(true);
        let meta = ArrowReaderMetadata::load_async(&mut reader, options).await?;
        Ok(Self { reader, meta })
    }

    pub fn num_row_groups(&self) -> usize {
        self.meta.metadata().num_row_groups()
    }

    fn builder(&self) -> ParquetRecordBatchStreamBuilder<R> {
        ParquetRecordBatchStreamBuilder::new_with_metadata(self.reader.clone(), self.meta.clone())
    }

    pub async fn read(&self, coord_type: &CoordType) -> Result<GeoTable> {
        let builder = self.builder();
        read_builder(builder, coord_type).await
    }

    pub async fn read_row_groups(
        &self,
        row_groups: Vec<usize>,
        coord_type: &CoordType,
    ) -> Result<GeoTable> {
        let builder = self.builder().with_row_groups(row_groups);
        read_builder(builder, coord_type).await
    }
}

pub struct ParquetDataset<R: AsyncFileReader + Clone + Unpin + Send + 'static> {
    files: Vec<ParquetFile<R>>,
}

impl<R: AsyncFileReader + Clone + Unpin + Send + 'static> ParquetDataset<R> {
    pub async fn read(&self, coord_type: &CoordType) -> Result<Vec<GeoTable>> {
        let futures = self.files.iter().map(|file| file.read(coord_type));
        let tables = futures::future::join_all(futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;
        Ok(tables)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::fs::File;

    #[tokio::test]
    async fn nybb() {
        let file = File::open("fixtures/geoparquet/nybb.parquet")
            .await
            .unwrap();
        let options = GeoParquetReaderOptions::new(65536, Default::default());
        let _output_geotable = read_geoparquet_async(file, options).await.unwrap();
    }
}
