use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_events::encodings::Body;
use http::header::HeaderMap;
use lambda_runtime::{handler_fn, Context, Error};
use log::LevelFilter;
use simple_logger::SimpleLogger;

use http::header::{CONNECTION, CONTENT_TYPE, CONTENT_LENGTH, TRANSFER_ENCODING};

use futures::StreamExt;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::page::{
    CaptureScreenshotFormat, CaptureScreenshotParams,
};

use std::fs;
use image;
use base64;
use reqwest;


#[tokio::main]
async fn main() -> Result<(), Error> {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();
    
    let func = handler_fn(my_handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

pub(crate) async fn my_handler(event: ApiGatewayProxyRequest, _ctx: Context) -> Result<ApiGatewayProxyResponse, Error> {
    

    // create a `Browser` that spawns a `chromium` process running with UI (`with_head()`, headless is default) 
    // and the handler that drives the websocket etc.
    let (browser, mut handler) =
    Browser::launch(BrowserConfig::builder().with_head().build()?).await?;

    // spawn a new task that continuously polls the handler
    let handle = tokio::task::spawn(async move {
    loop {
        let _ = handler.next().await.unwrap();
    }
    });
    let page = browser.new_page("https://en.wikipedia.org").await?;

    // take a screenshot of the page
    // page.find_element(".central-featured")
    //     .await?
    //     .save_screenshot(CaptureScreenshotFormat::Png, "example.png")
    //     .await?;
    let screenshot = page.screenshot(
        CaptureScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .build()
    ).await?;

    // page.save_screenshot(
    //     CaptureScreenshotParams::builder()
    //         .format(CaptureScreenshotFormat::Png)
    //         .build(),
    //     "hn-page.png",
    // )
    // .await?;

    // // let image = fs::read("hn-page.png");
    // let image_path = "hn-page.png";
    // let image_size = fs::metadata(image_path).unwrap().len();
    // let mut base_img = image::io::Reader::open("hn-page.png")?.decode()?;
    // let base_img = image::open(image_path)?;
    let base_img = image::load_from_memory(&screenshot)?;
    let image_size = screenshot.len();
    // let base_img_bytes = base_img.as_bytes();
    // let base_img_utf8 = String::from_utf8_lossy(base_img_bytes);

    let mut buf : Vec<u8> = Vec::new();
    base_img.write_to(&mut buf, image::ImageOutputFormat::Png)?;
    let res_base64 = base64::encode(&buf);

    // println!("{}", res_base64);
    // let image_b64 = image_base64::to_base64(image_path);

    // let res_base64 = image_base64::to_base64(image_path); 
    // let decoded_base64 = base64::decode(res_base64).unwrap();

    // let mut buff = Vec::new();
    // base64::decode_config_slice(res_base64, base64::STANDARD, &mut buff);
    // let base_img_utf8 = String::from_utf8_lossy(&res_base64);

    // let client = reqwest::Client::new();
    // let params = [("file", format!("data:image/png;base64,{}", res_base64)), ("upload_preset", "rqeiqymz".to_string())];
    // let request = client.post("https://api.cloudinary.com/v1_1/wl2ezqgs/image/upload")
    //     .form(&params)
    //     .send()
    //     .await?
    //     .json()
    //     .await?;

    // println!("body = {:?}", request.url);

    // let path = event.path.unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(CONNECTION, "keep-alive".parse().unwrap());
    headers.insert(CONTENT_TYPE, "image/png".parse().unwrap());
    // headers.insert(CONTENT_TYPE, "application/octet".parse().unwrap());
    // headers.insert(CONTENT_LENGTH, image_size.to_string().parse().unwrap());
    headers.insert(TRANSFER_ENCODING, "chunked".to_string().parse().unwrap());

    let resp = ApiGatewayProxyResponse {
        status_code: 200,
        headers: headers,
        multi_value_headers: HeaderMap::new(),
        // body: Some(Body::Text(res_base64.to_owned())),
        // body: Some(Body::Text(base_img_utf8.to_string())),
        body: Some(Body::Binary(screenshot)),
        is_base64_encoded: Some(true),
    };
    
    // handle.await;


    println!("Screenshots successfully created.");

    Ok(resp)
}