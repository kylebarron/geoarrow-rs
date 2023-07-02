use geozero::{GeomProcessor, GeozeroGeometry};

use crate::{LineStringArray, GeometryArrayTrait};

impl GeozeroGeometry for LineStringArray {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> geozero::error::Result<()>
    where
        Self: Sized,
    {
        let num_geometries = self.len();
        processor.geometrycollection_begin(num_geometries, 0)?;

        for geom_idx in 0..num_geometries {
            let (start_coord_idx, end_coord_idx) = self.geom_offsets.start_end(geom_idx);

            processor.linestring_begin(true, end_coord_idx - start_coord_idx, geom_idx)?;

            for coord_idx in start_coord_idx..end_coord_idx {
                processor.xy(
                    self.coords.get_x(coord_idx),
                    self.coords.get_y(coord_idx),
                    coord_idx - start_coord_idx,
                )?;
            }

            processor.linestring_end(true, geom_idx)?;
        }

        processor.geometrycollection_end(num_geometries - 1)?;
        Ok(())
    }
}
