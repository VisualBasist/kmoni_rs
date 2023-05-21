use serde::Deserialize;
use time::{macros::offset, Duration, OffsetDateTime, PrimitiveDateTime};

#[derive(Debug)]
pub struct KMoniClient {
    delay: Duration,
}

#[derive(Deserialize, Debug)]
struct ResultRawResponse {
    // TODO: enumにしたい
    status: String,
    message: String,
}

time::serde::format_description!(
    slash_separated_date,
    PrimitiveDateTime,
    "[year]/[month]/[day] [hour]:[minute]:[second]"
);

#[derive(Deserialize, Debug)]
struct LatestDataTimeResponse {
    #[serde(with = "slash_separated_date")]
    latest_time: PrimitiveDateTime,
    #[serde(with = "slash_separated_date")]
    request_time: PrimitiveDateTime,
    result: ResultRawResponse,
}

/*
{
    "result": {
        "status": "success",
        "message": "",
        "is_auth": true
    },
    "report_time": "2023/05/21 16:03:57",
    "region_code": "",
    "request_time": "20230521160357",
    "region_name": "福島県沖",
    "longitude": "141.5",
    "is_cancel": false,
    "depth": "20km",
    "calcintensity": "2",
    "is_final": false,
    "is_training": false,
    "latitude": "37.2",
    "origin_time": "20230521160321",
    "security": {
        "realm": "/kyoshin_monitor/static/jsondata/eew_est/",
        "hash": "b61e4d95a8c42e004665825c098a6de4"
    },
    "magunitude": "3.5",
    "report_num": "2",
    "request_hypo_type": "eew",
    "report_id": "20230521160327",
    "alertflg": "予報"
}
 */
#[derive(Deserialize, Debug)]
struct EEWRawResponse {
    result: ResultRawResponse,
    report_time: String,
    region_code: String,
    request_time: String,
    region_name: String,
    longitude: String,
    is_cancel: String,
    depth: String,
    calcintensity: String,
    is_final: String,
    is_training: String,
    latitude: String,
    origin_time: String,
    magunitude: String,
    report_num: String,
    request_hypo_type: String,
    report_id: String,
}

#[derive(Debug)]
pub struct EEW {
    // result: ResultRawResponse,
    report_time: OffsetDateTime,
    region_code: String,
    request_time: OffsetDateTime,
    region_name: String,
    longitude: f64,
    is_cancel: bool,
    depth: f64,
    // u32?
    calcintensity: f64,
    is_final: bool,
    is_training: bool,
    latitude: f64,
    origin_time: OffsetDateTime,
    magunitude: f64,
    report_num: u32,
    request_hypo_type: String,
    report_id: String,
}

impl KMoniClient {
    pub fn new() -> KMoniClient {
        // TODO: asyncにする?
        let response = reqwest::blocking::get(
            "http://www.kmoni.bosai.go.jp/webservice/server/pros/latest.json",
        )
        .unwrap()
        .json::<LatestDataTimeResponse>()
        .unwrap();

        KMoniClient {
            delay: time::OffsetDateTime::now_utc()
                - response.latest_time.assume_offset(offset!(+9)),
        }
    }

    pub fn fetch() {
        // TODO: async
        let response = reqwest::blocking::get(
            "http://www.kmoni.bosai.go.jp/webservice/hypo/eew/20230519003106.json",
        )
        .unwrap()
        .json::<EEWRawResponse>()
        .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(42, 42);
    }
}
