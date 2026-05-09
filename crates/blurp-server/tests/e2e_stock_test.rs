mod common;

use common::TestClient;

#[tokio::test]
async fn test_stock_reservation_on_checkout() {
    let client = TestClient::new();
    let token = client.login("stdtest2", "Test1234!").await;
    
    // Clear cart
    client.client
        .delete(format!("{}/api/cart", client.base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    
    // Get product with variant
    let products: serde_json::Value = client.client
        .get(format!("{}/api/catalog/products", client.base_url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    
    let product_id = products["data"][0]["id"].as_str().expect("No product");
    let product_detail: serde_json::Value = client.client
        .get(format!("{}/api/catalog/products/{}", client.base_url, product_id))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    
    let variant = &product_detail["data"]["variants"][0];
    let variant_id = variant["id"].as_str().expect("No variant");
    let initial_stock = variant["stock"].as_i64().expect("No stock");
    
    // Add 2 items to cart
    let res = client.client
        .post(format!("{}/api/cart/items", client.base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "variant_id": variant_id,
            "quantity": 2
        }))
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());
    
    // Checkout
    let order: serde_json::Value = client.client
        .post(format!("{}/api/orders/checkout", client.base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "shipping_address": {
                "name": "Stock Test",
                "phone": "+6281234567890",
                "street": "Jl. Test",
                "city": "Bandung",
                "province": "Jawa Barat",
                "postal_code": "40111",
                "country": "Indonesia",
                "district_id": 22
            },
            "courier_code": "jne",
            "service_code": "REG"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    
    assert_eq!(order["data"]["status"], "pending", "Order should be pending");
    
    // Verify stock reservation exists (stock should still be same, reserved not deducted)
    let product_after: serde_json::Value = client.client
        .get(format!("{}/api/catalog/products/{}", client.base_url, product_id))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    
    let stock_after = product_after["data"]["variants"][0]["stock"].as_i64().unwrap();
    
    // Stock should remain unchanged (reserved but not deducted until payment)
    assert_eq!(stock_after, initial_stock, "Stock should not be deducted until payment confirmed");
}

#[tokio::test]
async fn test_insufficient_stock_error() {
    let client = TestClient::new();
    let token = client.login("stdtest2", "Test1234!").await;
    
    // Clear cart
    client.client
        .delete(format!("{}/api/cart", client.base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    
    // Get product with variant
    let products: serde_json::Value = client.client
        .get(format!("{}/api/catalog/products", client.base_url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    
    let product_id = products["data"][0]["id"].as_str().expect("No product");
    let product_detail: serde_json::Value = client.client
        .get(format!("{}/api/catalog/products/{}", client.base_url, product_id))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    
    let variant_id = product_detail["data"]["variants"][0]["id"].as_str().expect("No variant");
    
    // Try to add way more than available stock
    let result: serde_json::Value = client.client
        .post(format!("{}/api/cart/items", client.base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "variant_id": variant_id,
            "quantity": 999999
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    
    // Cart add should fail with insufficient stock error
    assert!(result["error"].is_object(), "Should return error");
    assert!(result["error"]["message"].as_str().unwrap().contains("Insufficient stock"), 
            "Error should mention insufficient stock");
}
