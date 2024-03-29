use serde::{Deserialize, Deserializer};
use time::{
    macros::{format_description, offset},
    Duration, OffsetDateTime, PrimitiveDateTime,
};

#[derive(Debug)]
pub struct KMoniClient {
    delay: Duration,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
struct ResultRawResponse {
    // TODO: enumにしたい
    status: String,
    message: String,
}

// TODO: OffsetDataTimeを使う
time::serde::format_description!(
    slash_separated_date,
    PrimitiveDateTime,
    "[year]/[month]/[day] [hour]:[minute]:[second]"
);

time::serde::format_description!(
    nospace_date,
    PrimitiveDateTime,
    "[year][month][day][hour][minute][second]"
);

fn deserialize_string_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.parse().unwrap())
}

fn deserialize_string_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.parse().unwrap())
}

fn deserialize_km_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.strip_suffix("km")
        .expect("深さの単位がkmじゃない")
        .parse()
        .map_err(serde::de::Error::custom)
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
struct LatestDataTimeResponse {
    #[serde(with = "slash_separated_date")]
    latest_time: PrimitiveDateTime,
    #[serde(with = "slash_separated_date")]
    request_time: PrimitiveDateTime,
    result: ResultRawResponse,
}

#[derive(Deserialize, PartialEq, Eq)]
pub struct EEWOnlyResult {
    result: ResultRawResponse,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct EEW {
    result: ResultRawResponse,
    #[serde(with = "slash_separated_date")]
    report_time: PrimitiveDateTime,
    region_code: String,
    #[serde(with = "nospace_date")]
    request_time: PrimitiveDateTime,
    region_name: String,
    #[serde(deserialize_with = "deserialize_string_f64")]
    longitude: f64,
    is_cancel: bool,
    #[serde(deserialize_with = "deserialize_km_u32")]
    depth: u32,
    // TODO: calcintensityは5弱とかにもなる
    calcintensity: String,
    is_final: bool,
    is_training: bool,
    #[serde(deserialize_with = "deserialize_string_f64")]
    latitude: f64,
    #[serde(with = "nospace_date")]
    origin_time: PrimitiveDateTime,
    #[serde(deserialize_with = "deserialize_string_f64")]
    magunitude: f64,
    #[serde(deserialize_with = "deserialize_string_u32")]
    report_num: u32,
    request_hypo_type: String,
    report_id: String,
}

impl KMoniClient {
    pub async fn new() -> KMoniClient {
        let response =
            reqwest::get("http://www.kmoni.bosai.go.jp/webservice/server/pros/latest.json")
                .await
                .unwrap()
                .json::<LatestDataTimeResponse>()
                .await
                .unwrap();

        KMoniClient {
            delay: time::OffsetDateTime::now_utc()
                - response.latest_time.assume_offset(offset!(+9)),
        }
    }

    pub async fn fetch(&self) -> Option<EEW> {
        KMoniClient::parse_eew(&reqwest::get((OffsetDateTime::now_utc() - self.delay).to_offset(offset!(+9))
                .format(format_description!(
                    "http://www.kmoni.bosai.go.jp/webservice/hypo/eew/[year][month][day][hour][minute][second].json"
                ))
                .unwrap()
        )
        .await
        .unwrap().text().await.unwrap()).unwrap()
    }

    fn parse_eew(json: &str) -> serde_json::Result<Option<EEW>> {
        if serde_json::from_str::<EEWOnlyResult>(&json)?.result.message == "データがありません"
        {
            Ok(None)
        } else {
            Ok(Some(serde_json::from_str::<EEW>(&json)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::*;

    #[test]
    fn parse_succeeded() {
        assert_eq!(
            Some(EEW {
                result: ResultRawResponse {
                    status: "success".to_string(),
                    message: "".to_string()
                },
                report_time: datetime!(2023-05-21 16:03:57),
                region_code: "".to_string(),
                request_time: datetime!(2023-05-21 16:03:57),
                region_name: "福島県沖".to_string(),
                longitude: 141.5,
                is_cancel: false,
                depth: 20,
                calcintensity: "2".to_string(),
                is_final: false,
                is_training: false,
                latitude: 37.2,
                origin_time: datetime!(2023-05-21 16:03:21),
                magunitude: 3.5,
                report_num: 2,
                request_hypo_type: "eew".to_string(),
                report_id: "20230521160327".to_string()
            }),
            KMoniClient::parse_eew(
                r#"{
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
            }"#,
            )
            .unwrap()
        );
    }

    #[test]
    fn nodata() {
        assert_eq!(
            None,
            KMoniClient::parse_eew(
                r#"{
    "result": {
        "status": "success",
        "message": "データがありません",
        "is_auth": true
    },
    "report_time": "",
    "region_code": "",
    "request_time": "20240102114930",
    "region_name": "",
    "longitude": "",
    "is_cancel": "",
    "depth": "",
    "calcintensity": "",
    "is_final": "",
    "is_training": "",
    "latitude": "",
    "origin_time": "",
    "security": {
        "realm": "/webservice/hypo/eew/",
        "hash": "5ca8b8104e01ceef0f061ad597606cbd87b492db"
    },
    "magunitude": "",
    "report_num": "",
    "request_hypo_type": "eew",
    "report_id": ""
}"#,
            )
            .unwrap()
        );
    }
}
