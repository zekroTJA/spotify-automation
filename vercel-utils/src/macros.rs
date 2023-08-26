#[macro_export]
macro_rules! expect {
    ($expression:expr) => {
        match $expression {
            Ok(v) => v,
            Err(err) => {
                return vercel_runtime::http::internal_server_error(err.to_string());
            }
        }
    };

    ($expression:expr, $bail:expr) => {
        match $expression {
            Some(v) => v,
            None => {
                return $bail;
            }
        }
    };

    ($expression:expr, $( $pattern:pat_param $(if $guard:expr)? => $bail:expr),* ) => {
        match $expression {
            Ok(v) => v,
            $(
                $pattern $(if $guard)? => {
                    return $bail;
                }
            )*
        }
    };
}

#[macro_export]
macro_rules! get_path_param {
    ($req:expr, $key:expr) => {{
        let v = expect!(get_query_param($req, $key),
            Err(err) => vercel_runtime::http::internal_server_error(
                format!("failed parsing url: {err}")));
        expect!(v, internal_server_error(format!(
            "query params does not contain a value for {} - this should never happen", $key)))
    }};
}

#[macro_export]
macro_rules! method_handlers {
    ( $req:expr, $( $method:literal => $handler:expr ),* $(,)* ) => {
        match $req.method().to_string().as_str() {
            $(
                $method => $handler,
            )*
            _ => Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Body::Text("method not allowed".into()))?),
        }
    };
}
