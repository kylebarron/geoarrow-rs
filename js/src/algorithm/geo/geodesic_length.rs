use crate::array::*;
use wasm_bindgen::prelude::*;

macro_rules! impl_geodesic_length {
    ($struct_name:ident) => {
        #[wasm_bindgen]
        impl $struct_name {
            /// Determine the length of a geometry on an ellipsoidal model of the earth.
            ///
            /// This uses the geodesic measurement methods given by [Karney (2013)]. As opposed to
            /// older methods like Vincenty, this method is accurate to a few nanometers and always
            /// converges.
            ///
            /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
            #[wasm_bindgen]
            pub fn geodesic_length(&self) -> FloatArray {
                use geoarrow::algorithm::geo::GeodesicLength;
                FloatArray(GeodesicLength::geodesic_length(&self.0))
            }
        }
    };
}

impl_geodesic_length!(PointArray);
impl_geodesic_length!(MultiPointArray);
impl_geodesic_length!(LineStringArray);
impl_geodesic_length!(MultiLineStringArray);
