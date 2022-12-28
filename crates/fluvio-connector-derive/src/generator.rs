use proc_macro2::TokenStream;
use quote::quote;

use crate::ast::{ConnectorFn, ConnectorDirection};

pub(crate) fn generate_connector(direction: ConnectorDirection, func: &ConnectorFn) -> TokenStream {
    match direction {
        ConnectorDirection::Source => generate_source(func),
        ConnectorDirection::Sink => generate_sink(func),
    }
}

fn generate_source(func: &ConnectorFn) -> TokenStream {
    let user_fn = &func.name;
    let user_code = &func.func;

    let init_and_parse_config = init_and_parse_config();
    quote! {

        fn main() -> ::fluvio_connector_common::Result<()> {
            #init_and_parse_config

            ::fluvio_connector_common::future::run_block_on(async {
                let (fluvio, producer) = ::fluvio_connector_common::producer::producer_from_config(&config).await?;

                let metrics = ::std::sync::Arc::new(::fluvio_connector_common::monitoring::ConnectorMetrics::new(fluvio.metrics()));
                ::fluvio_connector_common::monitoring::init_monitoring(metrics);

                #user_fn(config, producer).await
            })?;

            Ok(())
        }

        #user_code
    }
}

fn generate_sink(func: &ConnectorFn) -> TokenStream {
    let user_fn = &func.name;
    let user_code = &func.func;

    let init_and_parse_config = init_and_parse_config();
    quote! {

        fn main() -> ::fluvio_connector_common::Result<()> {
            #init_and_parse_config

            ::fluvio_connector_common::future::run_block_on(async {
                let (fluvio, stream) = ::fluvio_connector_common::consumer::consumer_stream_from_config(&config).await?;

                let metrics = ::std::sync::Arc::new(::fluvio_connector_common::monitoring::ConnectorMetrics::new(fluvio.metrics()));
                ::fluvio_connector_common::monitoring::init_monitoring(metrics);

                #user_fn(config, stream).await
            })?;

            Ok(())
        }

        #user_code
    }
}

fn init_and_parse_config() -> TokenStream {
    quote! {
        #[derive(Debug)]
        pub struct ConnectorOpt {
            config: ::std::path::PathBuf,
        }

        impl ConnectorOpt {
            fn parse() -> Self {
                let path = ::std::env::args()
                    .enumerate()
                    .find(|(_, a)| a.eq("--config"))
                    .and_then(|(i, _)| ::std::env::args().nth(i + 1))
                    .map(::std::path::PathBuf::from);
                match path {
                    Some(config) => Self {config},
                    None => {
                        eprintln!("error: The following required arguments were not provided:\n  --config <PATH>");
                        ::std::process::exit(1)
                    }
                }
            }
        }

        ::fluvio_connector_common::future::init_logger();

        let opts = ConnectorOpt::parse();
        ::fluvio_connector_common::tracing::info!("Reading config file from: {}", opts.config.to_string_lossy());

        let config = ::fluvio_connector_common::ConnectorConfig::from_file(opts.config.as_path())?;
        ::fluvio_connector_common::tracing::debug!("{:#?}", config);

        ::fluvio_connector_common::tracing::info!("starting processing");
    }
}