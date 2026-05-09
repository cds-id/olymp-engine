mod common;

use common::TestClient;

#[tokio::test]
async fn test_cart_and_checkout_flow() {
    let client = TestClient::new();
    let token = client.login("stdtest2", "Test1234!").await;
    
    // 1. Clear cart
    let res = client.client
        .delete(format!("{}/api/cart", client.base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);
    
    // 2. Get or create product with variant
    let products: serde_json::Value = client.client
        .get(format!("{}/api/catalog/products", client.base_url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    
    let variant_id = if products["data"].as_array().map(|a| a.is_empty()).unwrap_or(true) {
        // Create category first
        let ts = chrono::Utc::now().timestamp();
        let category: serde_json::Value = client.client
            .post(format!("{}/api/catalog/categories", client.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({
                "name": format!("E2E Test Category {}", ts),
                "slug": format!("e2e-test-cat-{}", ts)
            }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        
        let category_id = category["data"]["id"].as_str().expect("Failed to create category");
        
        // Create test product
        let product: serde_json::Value = client.client
            .post(format!("{}/api/catalog/products", client.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({
                "name": format!("E2E Test Product {}", ts),
                "slug": format!("e2e-test-product-{}", ts),
                "description": "Test product for E2E checkout flow",
                "base_price_idr": 100000,
                "weight_grams": 500,
                "category_id": category_id,
                "sku": format!("E2E-PROD-{}", ts)
            }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        
        let product_id = product["data"]["id"].as_str().expect("Failed to create product");
        
        // Create variant
        let variant: serde_json::Value = client.client
            .post(format!("{}/api/catalog/products/{}/variants", client.base_url, product_id))
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({
                "name": "Default",
                "sku": format!("E2E-VAR-{}", ts),
                "price_idr": 100000,
                "stock": 100,
                "weight_grams": 500
            }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        
        variant["data"]["id"].as_str().expect("Failed to create variant").to_string()
    } else {
        // Get product detail to access variants
        let product_id = products["data"][0]["id"].as_str().expect("No product ID");
        let product_detail: serde_json::Value = client.client
            .get(format!("{}/api/catalog/products/{}", client.base_url, product_id))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        
        product_detail["data"]["variants"][0]["id"]
            .as_str()
            .expect("No variant found in product detail")
            .to_string()
    };
    
    // 3. Add to cart
    let res = client.client
        .post(format!("{}/api/cart/items", client.base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "variant_id": variant_id,
            "quantity": 1
        }))
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success(), "Add to cart failed: {}", res.status());
    
    // 4. Get shipping quote
    let quote: serde_json::Value = client.client
        .post(format!("{}/api/orders/shipping-quote", client.base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "shipping_address": {
                "name": "Test User",
                "phone": "+6281234567890",
                "street": "Jl. Test 123",
                "city": "Bandung",
                "province": "Jawa Barat",
                "postal_code": "40111",
                "country": "Indonesia",
                "district_id": 22
            }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    
    assert!(quote["data"]["shipping_options"].is_array(), "Expected shipping_options array");
    let options = quote["data"]["shipping_options"].as_array().unwrap();
    assert!(!options.is_empty(), "Expected at least one shipping option");
    
    let courier_code = quote["data"]["shipping_options"][0]["courier_code"]
        .as_str()
        .expect("Missing courier_code");
    let service_code = quote["data"]["shipping_options"][0]["service_code"]
        .as_str()
        .expect("Missing service_code");
    
    // 5. Checkout with selected courier
    let order: serde_json::Value = client.client
        .post(format!("{}/api/orders/checkout", client.base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "shipping_address": {
                "name": "Test User",
                "phone": "+6281234567890",
                "street": "Jl. Test 123",
                "city": "Bandung",
                "province": "Jawa Barat",
                "postal_code": "40111",
                "country": "Indonesia",
                "district_id": 22
            },
            "courier_code": courier_code,
            "service_code": service_code
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    
    assert_eq!(order["data"]["status"], "pending", "Order should be pending");
    assert_eq!(order["data"]["courier_code"], courier_code, "Courier code mismatch");
    assert!(order["data"]["order_number"].as_str().is_some(), "Missing order_number");
    assert!(order["data"]["total_idr"].as_i64().is_some(), "Missing total_idr");
}
