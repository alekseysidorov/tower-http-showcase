use std::{env, fs::File, path::PathBuf};

use eyre::{Context as _, bail};
use futures_util::{StreamExt as _, TryStreamExt as _};
use http::{Uri, uri::PathAndQuery};
use showcase_api::model::RenderTileResponse;
use showcase_renderer::{assemble_rgb, plan_tiles, write_png};
use tower::{BoxError, ServiceBuilder, util::BoxCloneSyncService};
use tower_http_client::{ResponseExt as _, ServiceExt as _, rewrite_uri::RewriteUriLayer};
use tower_reqwest::HttpClientLayer;

type WorkerClient =
    BoxCloneSyncService<http::Request<reqwest::Body>, http::Response<reqwest::Body>, BoxError>;

#[derive(Debug)]
struct Args {
    worker: Uri,
    width: u32,
    height: u32,
    tile_size: u32,
    max_iterations: u32,
    output: PathBuf,
}

impl Args {
    fn parse() -> eyre::Result<Self> {
        let mut args = Self {
            worker: "http://127.0.0.1:8080".parse()?,
            width: 1024,
            height: 1024,
            tile_size: 128,
            max_iterations: 500,
            output: PathBuf::from("target/mandelbrot.png"),
        };
        let mut values = env::args().skip(1);
        while let Some(flag) = values.next() {
            if flag == "--help" || flag == "-h" {
                println!(
                    "Usage: showcase-renderer [--worker URI] [--width N] [--height N] [--tile-size N] [--max-iterations N] [--output PATH]"
                );
                std::process::exit(0);
            }
            let value = values
                .next()
                .ok_or_else(|| eyre::eyre!("missing value for {flag}"))?;
            match flag.as_str() {
                "--worker" => args.worker = value.parse().wrap_err("invalid worker URI")?,
                "--width" => args.width = value.parse().wrap_err("invalid image width")?,
                "--height" => args.height = value.parse().wrap_err("invalid image height")?,
                "--tile-size" => args.tile_size = value.parse().wrap_err("invalid tile size")?,
                "--max-iterations" => {
                    args.max_iterations = value.parse().wrap_err("invalid iteration count")?;
                }
                "--output" => args.output = PathBuf::from(value),
                _ => bail!("unknown argument: {flag}"),
            }
        }

        if args.worker.scheme().is_none() || args.worker.authority().is_none() {
            bail!("--worker must be an absolute URI with a scheme and authority");
        }
        Ok(args)
    }
}

fn make_worker_client(origin: Uri) -> eyre::Result<WorkerClient> {
    let parts = origin.into_parts();
    let scheme = parts
        .scheme
        .ok_or_else(|| eyre::eyre!("worker URI must include a scheme"))?;
    let authority = parts
        .authority
        .ok_or_else(|| eyre::eyre!("worker URI must include an authority"))?;
    let service = ServiceBuilder::new()
        // Resolve relative HTTP client paths against the worker origin.
        .layer(RewriteUriLayer::new(move |uri: &Uri| {
            Uri::builder()
                .scheme(scheme.clone())
                .authority(authority.clone())
                .path_and_query(
                    uri.path_and_query()
                        .cloned()
                        .unwrap_or_else(|| PathAndQuery::from_static("/")),
                )
                .build()
                .map_err(BoxError::from)
        }))
        .map_err(BoxError::from)
        .layer(HttpClientLayer)
        .service(reqwest::Client::new());
    Ok(BoxCloneSyncService::new(service))
}

async fn request_tile(
    client: &mut WorkerClient,
    tile: showcase_renderer::TilePlan,
) -> eyre::Result<RenderTileResponse> {
    let response = client
        .post("/render")
        .json(&tile.request)?
        .send()
        .await
        .map_err(eyre::Report::msg)?;
    Ok(response.body_reader().json().await?)
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse()?;
    let tiles = plan_tiles(args.width, args.height, args.tile_size, args.max_iterations)?;
    let worker = make_worker_client(args.worker)?;

    let responses: Vec<_> = futures_util::stream::iter(tiles.iter().cloned())
        .map(|tile| {
            let mut worker = worker.clone();
            async move { request_tile(&mut worker, tile).await }
        })
        .buffer_unordered(16)
        .try_collect()
        .await?;

    let pixels = assemble_rgb(args.width, args.height, &tiles, &responses)?;
    if let Some(parent) = args
        .output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    write_png(
        File::create(&args.output)?,
        args.width,
        args.height,
        &pixels,
    )?;
    println!(
        "Rendered {}x{} image from {} tiles to {}",
        args.width,
        args.height,
        tiles.len(),
        args.output.display()
    );
    Ok(())
}
