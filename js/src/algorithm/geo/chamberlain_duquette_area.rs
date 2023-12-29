use crate::data::*;
use arrow_wasm::arrow1::data::Float64Data;
use wasm_bindgen::prelude::*;

macro_rules! impl_alg {
    ($struct_name:ident) => {
        #[wasm_bindgen]
        impl $struct_name {
            /// Calculate the unsigned approximate geodesic area of a `Geometry`.
            #[wasm_bindgen(js_name = chamberlainDuquetteUnsignedArea)]
            pub fn chamberlain_duquette_unsigned_area(&self) -> Float64Data {
                use geoarrow::algorithm::geo::ChamberlainDuquetteArea;
                ChamberlainDuquetteArea::chamberlain_duquette_unsigned_area(&self.0).into()
            }

            /// Calculate the signed approximate geodesic area of a `Geometry`.
            #[wasm_bindgen(js_name = chamberlainDuquetteSignedArea)]
            pub fn chamberlain_duquette_signed_area(&self) -> Float64Data {
                use geoarrow::algorithm::geo::ChamberlainDuquetteArea;
                ChamberlainDuquetteArea::chamberlain_duquette_signed_area(&self.0).into()
            }
        }
    };
}

impl_alg!(PointData);
impl_alg!(LineStringData);
impl_alg!(PolygonData);
impl_alg!(MultiPointData);
impl_alg!(MultiLineStringData);
impl_alg!(MultiPolygonData);
impl_alg!(MixedGeometryData);
impl_alg!(GeometryCollectionData);
