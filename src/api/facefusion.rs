#[allow(unused)]

use std::collections::HashMap;
use serde_json::Map;

use qx_rs_server::err::{Error, Result};

use crate::auth_v3;
use qx_rs_server::env::DEFAULT;


#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FuseFaceErr {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Message")]
    pub message: String,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FuseFaceResponse {
    #[serde(rename = "Error")]
    pub error: Option<FuseFaceErr>,
    #[serde(rename = "FusedImage")]
    pub fused_image: Option<String>,
    #[serde(rename = "RequestId")]
    pub request_id: String,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct FuseFaceResp {
    #[serde(rename = "Response")]
    pub response: FuseFaceResponse,
}


pub async fn facefusion(activity_id: &str, material_id: &str, region: &str, pic_base64: &String) -> Result<String> {
    let mut data = HashMap::<&str, serde_json::Value>::new();
    data.insert("ProjectId", activity_id.into());
    data.insert("ModelId", material_id.into());
    data.insert("RspImgType", "url".into());
    let mut merge_infos = Map::<String, serde_json::Value>::new();
    merge_infos.insert("Image".to_string(), serde_json::Value::String(pic_base64.clone()));
    let mut arr = Vec::new();
    arr.push(serde_json::Value::Object(merge_infos));
    data.insert("MergeInfos", serde_json::Value::Array(arr));
    let raw_json = auth_v3::post_v3(
        DEFAULT,
        &"facefusion.tencentcloudapi.com".to_string(), 
        &region.to_string(), 
        &"facefusion".to_string(), 
        &"FuseFace".to_string(),
        &"2022-09-27".to_string(),
        &data).await?;

    let resp = serde_json::from_str::<FuseFaceResp>(&raw_json).map_err(|err| {
        let err = format!("tencent.facefusion facefusion parse json failed:{:?}", err);
        tracing::error!(err);
        Error::ThirdPart(err)
    })?;
    if let Some(err) = resp.response.error {
        let err = format!("tencent.sms send_message failed:{:?}", err);
        tracing::error!(err);
        return Err(Error::ThirdPart(err));
    } else {
        if let Some(img) = resp.response.fused_image {
            Ok(img)
        } else {
            Err(Error::ThirdPart(format!("tencent.sms send_message failed")))
        }
    }
}
