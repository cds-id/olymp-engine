mod common;

use common::TestClient;

#[tokio::test]
async fn test_catalog_flow() {
    let client = TestClient::new();
    
    // 1. List products
    let res = client.client
        .get(format!("{}/api/catalog/products", client.base_url))
        .send()
        .await
        .unwrap();
    
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["data"].is_array());
    
    // 2. Get product detail
    if let Some(product_id) = body["data"][0]["id"].as_str() {
        let res = client.client
            .get(format!("{}/api/catalog/products/{}", client.base_url, product_id))
            .send()
            .await
            .unwrap();
        
        assert_eq!(res.status(), 200);
        let detail: serde_json::Value = res.json().await.unwrap();
        assert!(detail["data"]["variants"].is_array());
    }
}
