mod net_util;
use env_logger::Env;
use log::info;

fn main() {
  // Initialize global logger. Logger value can be set via the 'RUST_LOG' environment variable.
  env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

  let ip_neigh_vec = net_util::get_ip_neighbors().unwrap();
  info!("them neighhhs -> {:#?}", ip_neigh_vec);
}
