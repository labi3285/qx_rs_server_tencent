#[allow(unused)]

use std::collections::HashMap;
use std::fmt::Write;

use qx_rs_server::err::{Error, Result};
use qx_rs_server::time;

use qx_rs_server::req::req;

use hmac::{Hmac, Mac};  
use sha2::{Digest, Sha256};

use qx_rs_server::env;
use qx_rs_server::env::DEFAULT;


pub async fn post_v3(
    which_tencent: &'static str,
    host: &String,
    region: &String,
    service: &String,
    action: &String,
    version: &String,
    data: &HashMap::<&str, serde_json::Value>,
) -> Result<String> {
    let mut which = "TENCENT".to_string();
    if which_tencent != DEFAULT {
        which = format!("TENCENT.{}", which_tencent);
    }
    let secret_id = env::str(&format!("{}.SECRET_ID", which))?;
    let secret_key = env::str(&format!("{}.SECRET_KEY", which))?;
    let endpoint = format!("https://{}", host);
    let algorithm = "TC3-HMAC-SHA256".to_string();
    let date_time = time::now();
    let timestamp = date_time.timestamp();
    let timestamp_str = timestamp.to_string();
    let date = time::format(&date_time, &time::Pattern::Date); // "2019-02-25"
    let http_method = "POST";
    let canonical_uri = "/";
    let canonical_querystring = "";
    let content_type = "application/json; charset=utf-8".to_string();
    let canonical_headers = format!("content-type:{}\nhost:{}\nx-tc-action:{}\n", content_type, host, action.to_lowercase());
    let signed_headers = "content-type;host;x-tc-action";
    let payload = serde_json::json!(data).to_string();

    let hashed_request_payload = _sha256(&payload.to_string())?;
    let canonical_request = format!("{}\n{}\n{}\n{}\n{}\n{}", http_method, canonical_uri, canonical_querystring, canonical_headers, signed_headers, hashed_request_payload);
    let credential_scope = format!("{}/{}/tc3_request", date, service);
    let hashed_canonical_request = _sha256(&canonical_request)?;
    let string_to_sign = format!("{}\n{}\n{}\n{}", algorithm, timestamp, credential_scope, hashed_canonical_request);
    let secret_date_vec = _hmac_sha256_vec(date.as_bytes(), format!("TC3{}", secret_key).as_bytes())?;
    let secret_service_vec = _hmac_sha256_vec(service.as_bytes(), &secret_date_vec)?;
    let secret_signing_vec = _hmac_sha256_vec("tc3_request".as_bytes(), &secret_service_vec)?;
    let signature_vec = _hmac_sha256_vec(string_to_sign.as_bytes(), &secret_signing_vec)?;
    let signature = signature_vec.into_iter().map(|b| format!("{:02x}", b)).collect::<String>();
    let authorization = format!("{} Credential={}/{}, SignedHeaders={}, Signature={}", algorithm, secret_id, credential_scope, signed_headers, signature);

    let mut headers = Vec::<(&str, &String)>::new();
    headers.push(("Authorization", &authorization));
    headers.push(("Content-Type", &content_type));
    headers.push(("Host", host));
    headers.push(("X-TC-Action", &action));
    headers.push(("X-TC-Timestamp", &timestamp_str));
    headers.push(("X-TC-Version", &version));
    headers.push(("X-TC-Region", region));
    let resp = req::post_application_json(&endpoint, &Some(headers), &payload).await?;
    let text = resp.text().await.map_err(|err| {
        let err = format!("tencent v3 request failed:{:?}", err);
        tracing::error!(err);
        Error::ThirdPart(err)
    })?;
    Ok(text)
}


fn _sha256(text: &String) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());  
    let res = hasher.finalize();
    let mut str = String::with_capacity(res.len() * 2);  
    for byte in &res {  
        write!(&mut str, "{:02x}", byte).unwrap();  
    } 
    Ok(str)
}

fn _hmac_sha256_vec<'a>(data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if let Ok(mut hmac) = Hmac::<Sha256>::new_from_slice(key) {
        hmac.update(data);
        let res = hmac.finalize();
        let res = res.into_bytes().to_vec();
        Ok(res)
    } else {
        Err(Error::ThirdPart(format!("_hmac_sha256_vec failed")))
    }
}