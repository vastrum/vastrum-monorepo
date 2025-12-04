impl RPCServer {
    pub async fn start_indexer_server(&self) -> Result<(), Box<dyn std::error::Error>> {
        let app = self.router();

        let addr = format!("127.0.0.1:{}", 3000);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
    pub fn router(&self) -> Router {
        let cors = CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([header::CONTENT_TYPE]);

        let state = AppState {
            networking: self.networking.clone(),
            page_database: self.page_database.clone(),
            domain_database: self.domain_database.clone(),
        };

        return Router::new()
            .route("/submit/", post(RPCServer::submit_transaction))
            .route("/page/", post(RPCServer::get_page))
            .route("/getepochproof/", get(RPCServer::get_epoch_proof))
            .route("/", get(|| async { "root" }))
            .layer(cors)
            .with_state(state);
    }

    async fn submit_transaction(
        State(state): State<AppState>,
        axum::Json(input): axum::Json<SubmitTransactionPayload>,
    ) -> impl IntoResponse {
        let bytes = input.transaction_bytes;
        let transaction = Transaction::decode(&bytes).unwrap();
        println!("received transaction {:#?}", transaction);
        state.networking.broadcast_transaction(transaction);
        return StatusCode::OK;
    }
    async fn get_page(
        State(state): State<AppState>,
        axum::Json(input): axum::Json<GetPagePayload>,
    ) -> impl IntoResponse {
        //check input sizes todo dos protection
        //check read pow here? in future treat as mining  towards pool of rpc node
        println!("getting page:  page_route: {}", input.page_path);

        let Some(site_id_delim_pos) = input.page_path.find("/") else {
            return Json(PageResponse {
                content: "Page not found".to_string(),
                site_id: "".to_string(),
            });
        };

        let site_id = &input.page_path[0..site_id_delim_pos];

        let page_route = &input.page_path[site_id_delim_pos + 1..];

        println!("site id is {site_id} ");
        println!("page_route is {page_route} ");

        let domain_lookup = state.domain_database.read_site_data(site_id);
        println!("domain lookup result {domain_lookup:#?}");
        if let Some(domain) = domain_lookup {
            let page_content =
                state.page_database.read_page(domain.site_id.clone(), page_route.to_string());
            if let Some(page_content) = page_content {
                return Json(PageResponse {
                    content: page_content.content,
                    site_id: domain.site_id.to_string(),
                });
            } else {
                return Json(PageResponse {
                    content: "Page not found".to_string(),
                    site_id: "".to_string(),
                });
            }
        } else {
            if let Some(site_id) = Sha256Digest::from_string(site_id) {
                let page_content = state.page_database.read_page(site_id, page_route.to_string());

                if let Some(page_content) = page_content {
                    return Json(PageResponse {
                        content: page_content.content,
                        site_id: site_id.to_string(),
                    });
                }
            }
            return Json(PageResponse {
                content: "Page not found".to_string(),
                site_id: "".to_string(),
            });
        }
    }

    async fn get_epoch_proof(State(_state): State<AppState>) -> impl IntoResponse {
        //static epoch state for now
        return Json(PageResponse {
            content: "Static epoch state: Not implemented".to_string(),
            site_id: "".to_string(),
        });
    }
    pub fn new(networking: Arc<Networking>) -> RPCServer {
        return RPCServer {
            networking: networking,
            page_database: Arc::new(PageDatabase::new()),
            domain_database: Arc::new(DomainDatabase::new()),
        };
    }
}
pub struct RPCServer {
    networking: Arc<Networking>,
    page_database: Arc<PageDatabase>,
    domain_database: Arc<DomainDatabase>,
}
#[derive(Clone)]
struct AppState {
    networking: Arc<Networking>,
    page_database: Arc<PageDatabase>,
    domain_database: Arc<DomainDatabase>,
}

use axum::{
    Json, Router,
    extract::State,
    http::{Method, StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use shared_types::{
    borsh::BorshExt,
    crypto::sha256::Sha256Digest,
    types::{
        execution::transaction::Transaction,
        rpc::types::{GetPagePayload, PageResponse, SubmitTransactionPayload},
    },
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::{
    db::{domaindb::DomainDatabase, pagedb::PageDatabase},
    p2p::networking::Networking,
};
