use vastrum_asset_embed::{EmbedRequest, generate};

fn main() {
    generate(EmbedRequest {
        folder: "scaffolds/site",
        static_name: "SITE_TEMPLATE",
        out_file: "site_template.rs",
    });
    generate(EmbedRequest {
        folder: "scaffolds/eth_dapp",
        static_name: "ETH_DAPP_TEMPLATE",
        out_file: "eth_dapp_template.rs",
    });
}
