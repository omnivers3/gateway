use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
/// Expected error format from backing apis
pub struct Message {
    #[serde(alias="httpStatus")]
    pub http_status: u16,
    pub timestamp: String,
    pub service: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorContext {
    pub requested: String,
    #[serde(alias="httpStatus")]
    pub http_status: u16,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message2 {
    pub pages: HashMap<String, String>,
    pub objects: Vec<String>,
    pub errors: Vec<ErrorContext>,
}

#[cfg(test)]
mod test {
    use std::{ include_str };
    use super::{ Message, Message2 };

    #[test]
    fn parse_valid_encoded_message1_string_properly() {
        let status: u16 = 404;
        let timestamp: &str = "12341234";
        let service: &str = "foo_service";
        let message: &str = "sample_message";

        let input = include_str!("./samples/valid_message1.json");
        
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

    #[test]
    fn parse_valid_encoded_message2_string_properly() {
        let requested: &str = "sample_resource";
        let status: u16 = 404;
        let message: &str = "sample_message";
        
        let input = include_str!("./samples/valid_message2.json");

        match serde_json::from_str::<Message2>(&input) {
            Err (err) => assert!(false, "Error parsing: {:?}", err),
            Ok (actual) => {
                assert_eq!(requested, actual.errors[0].requested);
                assert_eq!(status, actual.errors[0].http_status);
                assert_eq!(message, actual.errors[0].message);
            }
        }
    }
}