use actix_web::http::StatusCode;
use anyhow::Result;
use actix_web::{get, App, HttpResponse, HttpServer, ResponseError, web};
use askama::Template;
use thiserror::Error;
use opencv::{self as cv, prelude::*};

fn take_photo() -> Result<()> { // Note, this is anyhow::Result
    // カメラを起動
    let mut cam = cv::videoio::VideoCapture::new(0, cv::videoio::CAP_ANY)?;
    let mut frame = Mat::default(); // カメラデータ保存用配列

    // カメラから画像を読み込む
    cam.read(&mut frame)?;

    // // 幅:2500、高さ:1500ピクセル部分のみ保存(画像をリサイズする場合)
    // frame = cv::core::Mat::roi(&frame, cv::core::Rect::new(0, 0, 2500, 1500))?;
    let file_name:&str = "live.jpeg";
    cv::imgcodecs::imwrite(file_name, &frame, &cv::core::Vector::default())?;
    Ok(())
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate { }

#[derive(Error, Debug)]
enum MyError {
    #[error("Failed to render HTML")]
    AskamaError(#[from] askama::Error),
}

impl ResponseError for MyError {}

#[get("/")]
async fn index() -> Result<HttpResponse, MyError> {
    let html:IndexTemplate = IndexTemplate {};
    let response_body: String = html.render()?;
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(response_body))
}

#[get("/camera")]
async fn take_pic() -> Result<HttpResponse, MyError> {
    let _ = take_photo();
    let image_content = web::block(|| std::fs::read("live.jpeg")).await.unwrap();
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("image/jpeg")
        .body(image_content.unwrap()))
}

#[actix_web::main]
async fn main() -> Result<(), actix_web::Error> {
    HttpServer::new(move || {
        App::new()
        .service(index)
        .service(take_pic)
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await?;
    Ok(())
}
