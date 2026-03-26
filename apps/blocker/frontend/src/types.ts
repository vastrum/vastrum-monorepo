export interface BlockSummary {
    height: number;
    block_hash: string;
    previous_block_hash: string;
    timestamp: number;
    tx_count: number;
    round: number;
}

export interface TxSummary {
    tx_hash: string;
    sender: string | null;
    tx_type: string;
    target_site: string | null;
    function_sig: string | null;
}

export interface TxDetail {
    tx_hash: string;
    block_height: number;
    tx_index: number;
    timestamp: number;
    sender: string | null;
    tx_type: string;
    target_site: string | null;
    nonce: string;
    recent_block_height: number;
    function_sig: string | null;
}

export interface SiteDetail {
    site_id: string;
    module_id: string | null;
    deploy_tx: string;
    block_height: number;
    domain: string | null;
    tx_count: number;
}

export interface DomainInfo {
    domain_name: string;
    site_id: string;
    block_height: number;
}
