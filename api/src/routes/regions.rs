use crate::request_outputs::{Region as RegionOutput, RegionsOutput};
use poem::{
    Error, handler,
    http::StatusCode,
    web::{Data, Json},
};
use std::sync::{Arc, Mutex};
use store::store::Store;

#[handler]
pub async fn get_all_regions(
    Data(store): Data<&Arc<Mutex<Store>>>,
) -> Result<Json<RegionsOutput>, Error> {
    let mut locked_store = store
        .lock()
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;
    let regions = locked_store
        .get_all_regions()
        .map_err(|_| Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Json(RegionsOutput {
        regions: regions
            .into_iter()
            .map(|region| RegionOutput {
                id: region.id,
                name: region.name,
            })
            .collect(),
    }))
}
