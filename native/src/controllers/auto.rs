use crate::{errors::Result, guards::authorized_controller::AuthorizedController};
use rocket::{http::Status, Route};

#[get("/mostplayed?<time_ranges>&<name>")]
async fn mostplayed(
    controller: AuthorizedController,
    time_ranges: String,
    name: Option<String>,
) -> Result<(Status, String)> {
    let time_ranges = time_ranges.split(',').map(str::trim);
    let name = name.as_deref().unwrap_or("Current Top Songs");

    let ids = controller
        .update_mostplayed_playlists(time_ranges, name)
        .await?;

    Ok((Status::Ok, format!("updated playlists: {}", ids.join(", "))))
}

#[get("/dwa?<dw_name>&<dwa_name>")]
async fn dwa(
    controller: AuthorizedController,
    dw_name: Option<String>,
    dwa_name: Option<String>,
) -> Result<(Status, String)> {
    let dw_name = dw_name.as_deref().unwrap_or("Discover Weekly");
    let dwa_name = dwa_name.unwrap_or(format!("{dw_name} Archive"));

    let id = controller
        .archive_discover_weekly(dw_name, dwa_name)
        .await?;

    Ok((Status::Ok, format!("updated playlist: {id}")))
}

#[get("/timeranges?<name>&<from>&<to>")]
async fn timeranges(
    controller: AuthorizedController,
    name: Option<String>,
    from: u32,
    to: u32,
) -> Result<(Status, String)> {
    if from >= to {
        return Ok((
            Status::BadRequest,
            "value for 'from' must be smaller than 'to'".into(),
        ));
    }

    let name = name.unwrap_or_else(|| format!("Songs from {} to {}", from, to));

    let id = controller.update_timerange_playlist(from..to, name).await?;

    Ok((Status::Ok, format!("updated playlist: {id}")))
}

pub fn routes() -> Vec<Route> {
    routes![mostplayed, dwa, timeranges]
}
