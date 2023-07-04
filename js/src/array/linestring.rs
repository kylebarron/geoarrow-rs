use geoarrow::array::GeometryArray;
use wasm_bindgen::prelude::*;

use crate::array::point::PointArray;
use crate::array::primitive::Float64Array;
use crate::error::WasmResult;

#[wasm_bindgen]
pub struct LineStringArray(pub(crate) geoarrow::array::LineStringArray);

#[wasm_bindgen]
impl LineStringArray {
    #[wasm_bindgen]
    pub fn area(&self) -> WasmResult<Float64Array> {
        use geoarrow::algorithm::geo::area;
        let out = area(GeometryArray::LineString(self.0.clone()))?;
        Ok(Float64Array(out))
    }

    #[wasm_bindgen]
    pub fn center(&self) -> WasmResult<PointArray> {
        use geoarrow::algorithm::geo::center;
        let out = center(&GeometryArray::LineString(self.0.clone()))?;
        Ok(PointArray(out))
    }

    #[wasm_bindgen]
    pub fn signed_area(&self) -> WasmResult<Float64Array> {
        use geoarrow::algorithm::geo::signed_area;
        let out = signed_area(GeometryArray::LineString(self.0.clone()))?;
        Ok(Float64Array(out))
    }
}
