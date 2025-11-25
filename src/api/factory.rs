use super::fallback::FallbackClient;
use super::mtgio::MtgIoClient;
use super::scryfall::ScryfallClient;
use super::traits::CardApi;

/// API provider selection
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ApiProvider {
    #[default]
    Scryfall,
    MtgIo,
}

impl ApiProvider {
    pub fn name(&self) -> &'static str {
        match self {
            ApiProvider::Scryfall => "Scryfall",
            ApiProvider::MtgIo => "MTG.io",
        }
    }
}

/// Create a CardApi client based on provider selection
pub fn create_client(provider: ApiProvider, enable_fallback: bool) -> Box<dyn CardApi> {
    if enable_fallback {
        let (primary, fallback): (Box<dyn CardApi>, Box<dyn CardApi>) = match provider {
            ApiProvider::Scryfall => (
                Box::new(ScryfallClient::new()),
                Box::new(MtgIoClient::new()),
            ),
            ApiProvider::MtgIo => (
                Box::new(MtgIoClient::new()),
                Box::new(ScryfallClient::new()),
            ),
        };
        Box::new(FallbackClient::with_fallback(primary, fallback))
    } else {
        match provider {
            ApiProvider::Scryfall => Box::new(ScryfallClient::new()),
            ApiProvider::MtgIo => Box::new(MtgIoClient::new()),
        }
    }
}
