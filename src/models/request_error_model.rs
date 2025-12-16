use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ReqErrModel {
    pub message: String,
}

// impl ReqErrModel {
//     pub fn id(id: String) -> ReqErrModel {
//         ReqErrModel {
//             message: format!("Invalid id please try other id but not this one: {} ", id),
//         }
//     }
// }
