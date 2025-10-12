use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::utils::DurationSeconds;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssumeRoleAction {
    AssumeRoleWithWebIdentity,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssumeRoleWithWebIdentityRequest {
    pub action: AssumeRoleAction,
    pub role_session_name: String,
    pub role_arn: String,
    pub web_identity_token: String,
    /// STS API version
    ///
    /// Currently: `2011-06-15`
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<DurationSeconds>,
}

impl Default for AssumeRoleWithWebIdentityRequest {
    fn default() -> Self {
        Self {
            action: AssumeRoleAction::AssumeRoleWithWebIdentity,
            role_session_name: "aws-creds".to_string(),
            role_arn: Default::default(),
            web_identity_token: Default::default(),
            version: "2011-06-15".to_string(),
            duration_seconds: None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct AssumeRoleWithWebIdentityResponse {
    pub assume_role_with_web_identity_result: AssumeRoleWithWebIdentityResult,
    pub response_metadata: ResponseMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct AssumeRoleWithWebIdentityResult {
    pub subject_from_web_identity_token: String,
    pub audience: String,
    pub assumed_role_user: AssumedRoleUser,
    pub credentials: StsResponseCredentials,
    pub source_identity: Option<String>,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct AssumedRoleUser {
    pub arn: String,
    pub assumed_role_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct StsResponseCredentials {
    pub session_token: String,
    pub secret_access_key: String,
    pub expiration: DateTime<FixedOffset>,
    pub access_key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct ResponseMetadata {
    pub request_id: String,
}

#[cfg(test)]
mod tests {
    use crate::credentials::AssumeRoleWithWebIdentityResponse;

    #[test]
    fn parse_example_response() {
        let xml = r#"
<AssumeRoleWithWebIdentityResponse xmlns="https://sts.amazonaws.com/doc/2011-06-15/">
  <AssumeRoleWithWebIdentityResult>
    <SubjectFromWebIdentityToken>amzn1.account.AF6RHO7KZU5XRVQJGXK6HB56KR2A</SubjectFromWebIdentityToken>
    <Audience>client.5498841531868486423.1548@apps.example.com</Audience>
    <AssumedRoleUser>
      <Arn>arn:aws:sts::123456789012:assumed-role/FederatedWebIdentityRole/app1</Arn>
      <AssumedRoleId>AROACLKWSDQRAOEXAMPLE:app1</AssumedRoleId>
    </AssumedRoleUser>
    <Credentials>
      <SessionToken>AQoDYXdzEE0a8ANXXXXXXXXNO1ewxE5TijQyp+IEXAMPLE</SessionToken>
      <SecretAccessKey>wJalrXUtnFEMI/K7MDENG/bPxRfiCYzEXAMPLEKEY</SecretAccessKey>
      <Expiration>2014-10-24T23:00:23Z</Expiration>
      <AccessKeyId>ASgeIAIOSFODNN7EXAMPLE</AccessKeyId>
    </Credentials>
    <SourceIdentity>SourceIdentityValue</SourceIdentity>
    <Provider>www.amazon.com</Provider>
  </AssumeRoleWithWebIdentityResult>
  <ResponseMetadata>
    <RequestId>ad4156e9-bce1-11e2-82e6-6b6efEXAMPLE</RequestId>
  </ResponseMetadata>
</AssumeRoleWithWebIdentityResponse>
"#;

        let parsed: AssumeRoleWithWebIdentityResponse = quick_xml::de::from_str(xml).unwrap();
        println!("{:#?}", parsed);
    }
}
