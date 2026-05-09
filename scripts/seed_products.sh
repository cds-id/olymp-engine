#!/bin/bash
set -e

API_BASE="http://localhost:8080/api"
ADMIN_EMAIL="admin@sorastore.com"
ADMIN_PASSWORD="Admin123!"

echo "=== Blurp Product Seeder ==="

# Login as admin
echo "Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_BASE/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"'"$ADMIN_PASSWORD"'"}')

TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.data.access_token')

if [ "$TOKEN" = "null" ] || [ -z "$TOKEN" ]; then
  echo "Login failed. Creating admin user..."
  
  # Register admin
  REGISTER_RESPONSE=$(curl -s -X POST "$API_BASE/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"$ADMIN_EMAIL\",\"username\":\"admin\",\"password\":\"$ADMIN_PASSWORD\",\"name\":\"Admin User\"}")
  
  TOKEN=$(echo $REGISTER_RESPONSE | jq -r '.data.access_token')
  
  if [ "$TOKEN" = "null" ] || [ -z "$TOKEN" ]; then
    echo "Failed to create admin user"
    exit 1
  fi
fi

echo "Logged in successfully"

# Wipe existing products
echo "Wiping existing products..."
PRODUCTS=$(curl -s "$API_BASE/catalog/products" | jq -r '.data[].id')
for pid in $PRODUCTS; do
  curl -s -X DELETE "$API_BASE/catalog/products/$pid" \
    -H "Authorization: Bearer $TOKEN" > /dev/null
  echo "  Deleted product $pid"
done

# Wipe existing categories
echo "Wiping existing categories..."
CATEGORIES=$(curl -s "$API_BASE/catalog/categories" | jq -r '.data[].id')
for cid in $CATEGORIES; do
  curl -s -X DELETE "$API_BASE/catalog/categories/$cid" \
    -H "Authorization: Bearer $TOKEN" > /dev/null
  echo "  Deleted category $cid"
done

echo ""
echo "=== Creating Categories ==="

# Create Electronics category
ELECTRONICS=$(curl -s -X POST "$API_BASE/catalog/categories" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Electronics",
    "slug": "electronics",
    "description": "Electronic devices and accessories"
  }' | jq -r '.data.id')
echo "Created Electronics: $ELECTRONICS"

# Create Smartphones subcategory
SMARTPHONES=$(curl -s -X POST "$API_BASE/catalog/categories" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"Smartphones\",
    \"slug\": \"smartphones\",
    \"description\": \"Mobile phones and accessories\",
    \"parent_id\": \"$ELECTRONICS\"
  }" | jq -r '.data.id')
echo "Created Smartphones: $SMARTPHONES"

# Create Laptops subcategory
LAPTOPS=$(curl -s -X POST "$API_BASE/catalog/categories" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"Laptops\",
    \"slug\": \"laptops\",
    \"description\": \"Portable computers\",
    \"parent_id\": \"$ELECTRONICS\"
  }" | jq -r '.data.id')
echo "Created Laptops: $LAPTOPS"

# Create Fashion category
FASHION=$(curl -s -X POST "$API_BASE/catalog/categories" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Fashion",
    "slug": "fashion",
    "description": "Clothing and accessories"
  }' | jq -r '.data.id')
echo "Created Fashion: $FASHION"

# Create Men subcategory
MEN=$(curl -s -X POST "$API_BASE/catalog/categories" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"Men\",
    \"slug\": \"men\",
    \"description\": \"Men's clothing\",
    \"parent_id\": \"$FASHION\"
  }" | jq -r '.data.id')
echo "Created Men: $MEN"

# Create Women subcategory
WOMEN=$(curl -s -X POST "$API_BASE/catalog/categories" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"Women\",
    \"slug\": \"women\",
    \"description\": \"Women's clothing\",
    \"parent_id\": \"$FASHION\"
  }" | jq -r '.data.id')
echo "Created Women: $WOMEN"

echo ""
echo "=== Creating Products ==="

# Product 1: iPhone 15 Pro
echo "Creating iPhone 15 Pro..."
IPHONE=$(curl -s -X POST "$API_BASE/catalog/products" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"category_id\": \"$SMARTPHONES\",
    \"name\": \"iPhone 15 Pro\",
    \"slug\": \"iphone-15-pro\",
    \"description\": \"Latest iPhone with A17 Pro chip, titanium design, and advanced camera system\",
    \"base_price_idr\": 18999000,
    \"weight_grams\": 187,
    \"sku\": \"APPLE-IP15PRO\",
    \"length_mm\": 147,
    \"width_mm\": 71,
    \"height_mm\": 8
  }" | jq -r '.data.id')
echo "  Created product: $IPHONE"

# iPhone variants
curl -s -X POST "$API_BASE/catalog/products/$IPHONE/variants" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "128GB Natural Titanium",
    "sku": "IP15PRO-128-NAT",
    "price_idr": 18999000,
    "stock": 50,
    "attributes": [
      {"key": "Storage", "value": "128GB"},
      {"key": "Color", "value": "Natural Titanium"}
    ]
  }' > /dev/null

curl -s -X POST "$API_BASE/catalog/products/$IPHONE/variants" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "256GB Blue Titanium",
    "sku": "IP15PRO-256-BLU",
    "price_idr": 21999000,
    "stock": 35,
    "attributes": [
      {"key": "Storage", "value": "256GB"},
      {"key": "Color", "value": "Blue Titanium"}
    ]
  }' > /dev/null

curl -s -X POST "$API_BASE/catalog/products/$IPHONE/variants" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "512GB Black Titanium",
    "sku": "IP15PRO-512-BLK",
    "price_idr": 25999000,
    "stock": 20,
    "attributes": [
      {"key": "Storage", "value": "512GB"},
      {"key": "Color", "value": "Black Titanium"}
    ]
  }' > /dev/null

echo "  Added 3 variants"

# Product 2: Samsung Galaxy S24 Ultra
echo "Creating Samsung Galaxy S24 Ultra..."
SAMSUNG=$(curl -s -X POST "$API_BASE/catalog/products" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"category_id\": \"$SMARTPHONES\",
    \"name\": \"Samsung Galaxy S24 Ultra\",
    \"slug\": \"samsung-galaxy-s24-ultra\",
    \"description\": \"Flagship Android phone with S Pen, 200MP camera, and AI features\",
    \"base_price_idr\": 19999000,
    \"weight_grams\": 232,
    \"sku\": \"SAMSUNG-S24U\",
    \"length_mm\": 162,
    \"width_mm\": 79,
    \"height_mm\": 9
  }" | jq -r '.data.id')
echo "  Created product: $SAMSUNG"

curl -s -X POST "$API_BASE/catalog/products/$SAMSUNG/variants" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "256GB Titanium Gray",
    "sku": "S24U-256-GRAY",
    "price_idr": 19999000,
    "stock": 40,
    "attributes": [
      {"key": "Storage", "value": "256GB"},
      {"key": "Color", "value": "Titanium Gray"}
    ]
  }' > /dev/null

curl -s -X POST "$API_BASE/catalog/products/$SAMSUNG/variants" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "512GB Titanium Violet",
    "sku": "S24U-512-VIOLET",
    "price_idr": 23999000,
    "stock": 25,
    "attributes": [
      {"key": "Storage", "value": "512GB"},
      {"key": "Color", "value": "Titanium Violet"}
    ]
  }' > /dev/null

echo "  Added 2 variants"

# Product 3: MacBook Pro 14"
echo "Creating MacBook Pro 14..."
MACBOOK=$(curl -s -X POST "$API_BASE/catalog/products" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"category_id\": \"$LAPTOPS\",
    \"name\": \"MacBook Pro 14-inch M3\",
    \"slug\": \"macbook-pro-14-m3\",
    \"description\": \"Powerful laptop with M3 chip, Liquid Retina XDR display, and all-day battery\",
    \"base_price_idr\": 29999000,
    \"weight_grams\": 1600,
    \"sku\": \"APPLE-MBP14-M3\",
    \"length_mm\": 312,
    \"width_mm\": 221,
    \"height_mm\": 16
  }" | jq -r '.data.id')
echo "  Created product: $MACBOOK"

curl -s -X POST "$API_BASE/catalog/products/$MACBOOK/variants" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "M3 8-core CPU, 10-core GPU, 8GB RAM, 512GB SSD",
    "sku": "MBP14-M3-8-512",
    "price_idr": 29999000,
    "stock": 15,
    "weight_grams": 1600,
    "attributes": [
      {"key": "Chip", "value": "M3"},
      {"key": "RAM", "value": "8GB"},
      {"key": "Storage", "value": "512GB"}
    ]
  }' > /dev/null

curl -s -X POST "$API_BASE/catalog/products/$MACBOOK/variants" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "M3 Pro 11-core CPU, 14-core GPU, 18GB RAM, 1TB SSD",
    "sku": "MBP14-M3PRO-18-1TB",
    "price_idr": 39999000,
    "stock": 10,
    "weight_grams": 1600,
    "attributes": [
      {"key": "Chip", "value": "M3 Pro"},
      {"key": "RAM", "value": "18GB"},
      {"key": "Storage", "value": "1TB"}
    ]
  }' > /dev/null

echo "  Added 2 variants"

# Product 4: Men's T-Shirt
echo "Creating Men's Premium T-Shirt..."
TSHIRT=$(curl -s -X POST "$API_BASE/catalog/products" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"category_id\": \"$MEN\",
    \"name\": \"Premium Cotton T-Shirt\",
    \"slug\": \"mens-premium-cotton-tshirt\",
    \"description\": \"100% organic cotton, comfortable fit, perfect for everyday wear\",
    \"base_price_idr\": 199000,
    \"weight_grams\": 200,
    \"sku\": \"TSHIRT-MEN-PREM\",
    \"length_mm\": 300,
    \"width_mm\": 200,
    \"height_mm\": 20
  }" | jq -r '.data.id')
echo "  Created product: $TSHIRT"

for size in S M L XL; do
  for color in Black White Navy; do
    curl -s -X POST "$API_BASE/catalog/products/$TSHIRT/variants" \
      -H "Authorization: Bearer $TOKEN" \
      -H "Content-Type: application/json" \
      -d "{
        \"name\": \"$size $color\",
        \"sku\": \"TSHIRT-$size-${color:0:3}\",
        \"price_idr\": 199000,
        \"stock\": 100,
        \"attributes\": [
          {\"key\": \"Size\", \"value\": \"$size\"},
          {\"key\": \"Color\", \"value\": \"$color\"}
        ]
      }" > /dev/null
  done
done

echo "  Added 12 variants (4 sizes × 3 colors)"

# Product 5: Women's Dress
echo "Creating Women's Summer Dress..."
DRESS=$(curl -s -X POST "$API_BASE/catalog/products" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"category_id\": \"$WOMEN\",
    \"name\": \"Floral Summer Dress\",
    \"slug\": \"womens-floral-summer-dress\",
    \"description\": \"Light and breezy summer dress with floral pattern, perfect for warm weather\",
    \"base_price_idr\": 399000,
    \"weight_grams\": 250,
    \"sku\": \"DRESS-WOMEN-FLORAL\",
    \"length_mm\": 350,
    \"width_mm\": 250,
    \"height_mm\": 30
  }" | jq -r '.data.id')
echo "  Created product: $DRESS"

for size in S M L; do
  for color in "Blue Floral" "Pink Floral" "Yellow Floral"; do
    curl -s -X POST "$API_BASE/catalog/products/$DRESS/variants" \
      -H "Authorization: Bearer $TOKEN" \
      -H "Content-Type: application/json" \
      -d "{
        \"name\": \"$size $color\",
        \"sku\": \"DRESS-$size-${color:0:3}\",
        \"price_idr\": 399000,
        \"stock\": 50,
        \"attributes\": [
          {\"key\": \"Size\", \"value\": \"$size\"},
          {\"key\": \"Pattern\", \"value\": \"$color\"}
        ]
      }" > /dev/null
  done
done

echo "  Added 9 variants (3 sizes × 3 patterns)"

# Product 6: Wireless Earbuds
echo "Creating Wireless Earbuds..."
EARBUDS=$(curl -s -X POST "$API_BASE/catalog/products" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"category_id\": \"$ELECTRONICS\",
    \"name\": \"Pro Wireless Earbuds\",
    \"slug\": \"pro-wireless-earbuds\",
    \"description\": \"Active noise cancellation, 30-hour battery life, premium sound quality\",
    \"base_price_idr\": 1499000,
    \"weight_grams\": 50,
    \"sku\": \"EARBUDS-PRO\",
    \"length_mm\": 60,
    \"width_mm\": 50,
    \"height_mm\": 30
  }" | jq -r '.data.id')
echo "  Created product: $EARBUDS"

curl -s -X POST "$API_BASE/catalog/products/$EARBUDS/variants" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Matte Black",
    "sku": "EARBUDS-BLK",
    "price_idr": 1499000,
    "stock": 80,
    "attributes": [
      {"key": "Color", "value": "Matte Black"}
    ]
  }' > /dev/null

curl -s -X POST "$API_BASE/catalog/products/$EARBUDS/variants" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Pearl White",
    "sku": "EARBUDS-WHT",
    "price_idr": 1499000,
    "stock": 60,
    "attributes": [
      {"key": "Color", "value": "Pearl White"}
    ]
  }' > /dev/null

echo "  Added 2 variants"

echo ""
echo "=== Seed Complete ==="
echo "Created:"
echo "  - 6 categories (2 parent, 4 subcategories)"
echo "  - 6 products"
echo "  - 30 total variants"
echo ""
echo "View at: http://localhost:8080/swagger-ui"
