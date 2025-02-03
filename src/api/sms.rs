#[allow(unused)]

use std::collections::HashMap;

use qx_rs_server::err::{Error, Result};
use qx_rs_server::env::DEFAULT;

use crate::auth_v3;


#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SmsErr {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Message")]
    pub message: String,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SmsStatus {
    #[serde(rename = "SerialNo")]
    pub serial_no: String,
    #[serde(rename = "PhoneNumber")]
    pub phone_number: String,
    #[serde(rename = "Fee")]
    pub fee: i32,
    #[serde(rename = "SessionContext")]
    pub session_context: String,
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Message")]
    pub message: String,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SmsResponse {
    #[serde(rename = "Error")]
    pub error: Option<SmsErr>,
    #[serde(rename = "SendStatusSet")]
    pub send_status_set: Option<Vec<SmsStatus>>,
    #[serde(rename = "RequestId")]
    pub request_id: String,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct SmsResp {
    #[serde(rename = "Response")]
    pub response: SmsResponse,
}


pub async fn send_message(sms_sdk_app_id: &str, region: &str, sign_name: &str, phone: &String, template_id: &str, template_params: &Option<Vec<String>>) -> Result<()> {
    let mut data = HashMap::<&str, serde_json::Value>::new();
    data.insert("SmsSdkAppId", sms_sdk_app_id.into());
    data.insert("SignName", sign_name.into());
    data.insert("TemplateId", template_id.into());
    let mut phones = Vec::new();
    phones.push(serde_json::Value::String(phone.clone()));
    data.insert("PhoneNumberSet", serde_json::Value::Array(phones));
    let mut params = Vec::new();
    if let Some(arr) = template_params {
        for e in arr {
            params.push(serde_json::Value::String(e.clone()));
        }
    }
    data.insert("TemplateParamSet", serde_json::Value::Array(params));

    let raw_json = auth_v3::post_v3(
        DEFAULT,
        &"sms.tencentcloudapi.com".to_string(), 
        &region.to_string(), 
        &"sms".to_string(), 
        &"SendSms".to_string(),
        &"2021-01-11".to_string(),
        &data).await?;

    let resp = serde_json::from_str::<SmsResp>(&raw_json).map_err(|err| {
        let err = format!("tencent.sms send_message parse json failed:{:?}", err);
        tracing::error!(err);
        Error::ThirdPart(err)
    })?;
    if let Some(err) = resp.response.error {
        let err = format!("tencent.sms send_message failed:{:?}", err);
        tracing::error!(err);
        return Err(Error::ThirdPart(err));
    } else {
        if let Some(arr) = resp.response.send_status_set {
            if arr.len() > 0 {
                if arr[0].code == "Ok" {
                    return Ok(())
                } else {
                    let err = format!("tencent.sms send_message failed:({:?}){:?}", &arr[0].code, &arr[0].message);
                    return Err(Error::ThirdPart(err))
                }
            }
        }
    }
    return Err(Error::ThirdPart(format!("tencent.sms send_message failed")))
}
