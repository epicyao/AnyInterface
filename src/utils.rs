use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Response<T> {
    pub ok: bool,
    pub code: i32,
    pub data: T,
}

pub fn data_trans<T>(ok: bool, code: i32, data: T) -> Response<T> {
    Response { ok, code, data }
}

#[cfg(test)]
mod tests {
    use super::{Response, data_trans};

    #[test]
    fn builds_a_standard_response() {
        let response = data_trans(true, 200, "created");

        assert_eq!(
            response,
            Response {
                ok: true,
                code: 200,
                data: "created",
            }
        );
    }
}
