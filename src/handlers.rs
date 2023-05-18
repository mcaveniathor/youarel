use crate::{AppState, encoding::encode,ShortenReq};
use axum::{
    http::StatusCode, Form,
    response::{ Html, Redirect, Response, IntoResponse},
    extract::{State,Path},
};

const HEAD: &'static str = r#"
<!doctype html>
    <html lang="en">
        <head>
         <meta name="viewport" content="width=device-width, initial-scale=1">
        <!-- FONT
        –––––––––––––––––––––––––––––––––––––––––––––––––– -->
        <link href="//fonts.googleapis.com/css?family=Raleway:400,300,600" rel="stylesheet" type="text/css">

        <link rel="stylesheet" href="assets/css/skeleton-main.css">
        <link rel="stylesheet" href="assets/css/skeleton-dark.css">
        </head>
"#;



pub async fn redirect(State(state): State<AppState>, Path(key): Path<String>) -> Response {
    match state.db.get(key) {
        Ok(Some(long_url)) => {
            match String::from_utf8(long_url.to_vec()) {
                Ok(u_str) => Redirect::to(&u_str).into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        Ok(None) => {
            StatusCode::NOT_FOUND.into_response()

        },
        Err(_) => {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        },
    }
}

pub async fn accept_form(State(state): State<AppState>, Form(shorten_req): Form<ShortenReq>) -> Response {
    let long_s = shorten_req.long_url.as_str();
    let encoded = encode(long_s, shorten_req.length);
    match state.db.insert(&encoded, long_s) {
        Ok(_) => {
            let mut htmlstr = HEAD.to_string();
                htmlstr += r#"
               <body>
                <div class="container">
                    <div class="row">
                        <div class="six columns">
                            <h2>Short URL:</h2>
                        </div>
                        <div class="six columns">
                        "#;

            htmlstr += &format!("https://{}:{}/",state.hostname, state.port);
            htmlstr += &encoded;
            htmlstr += r#"
                        </div>
                    </div>
                </div>
                </body>
            </html>
            "#;
        Html(htmlstr).into_response()
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
pub async fn root() -> Html<String> {
    let mut htmlstr = HEAD.to_string();
    htmlstr += r#"
            <body>
            <div class="container">
                <form action="/" method="post">
                <div class="row">
                    <div class="twelve columns">
                        <label for="long_url">
                            Long URL
                            <input type="text" name="long_url" id="long_url">
                        </label>
                    </div>
                </div>
                <div class="row">
                    <input class="button-primary" type="submit" value="Shorten">
                <div>
                </form>
            </div>
            </body>
        </html>
        "#;
    Html(htmlstr)
}

