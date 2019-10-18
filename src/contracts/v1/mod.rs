// use serde_derive;

#[derive(Serialize, Deserialize, Debug)]
/// Expected error format from backing apis
pub struct Message {
    #[serde(alias="httpStatus")]
    pub http_status: u16,
    pub timestamp: String,
    pub service: String,
    pub message: String,
}

#[cfg(test)]
mod test {
    use super::{ Message };

    fn valid_error_response(status: u16, timestamp: &str, service: &str, message: &str) -> String {
        format!("{{\"httpStatus\":{},\"timestamp\":\"{}\",\"service\":\"{}\",\"message\":\"{}\"}}",
            status,
            timestamp,
            service,
            message
        )
    }

    #[test]
    fn parse_valid_encoded_string_properly() {
        let status: u16 = 401;
        let timestamp: &str = "12341234";
        let service: &str = "foo_service";
        let message: &str = "sample_message";

        let input = valid_error_response(status, timestamp, service, message);

        match serde_json::from_str::<Message>(&input) {
            Err (err) => assert!(false, "Error parsing: {:?}", err),
            Ok (actual) => {
                assert_eq!(status, actual.http_status);
                assert_eq!(timestamp, actual.timestamp);
                assert_eq!(service, actual.service);
                assert_eq!(message, actual.message);
            }
        }
    }
}