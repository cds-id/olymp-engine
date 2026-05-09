-- Catalog Seed Data: Products with Variants and Images
-- Uses Picsum.photos for placeholder images
-- Run with: psql $DATABASE_URL -f scripts/seed_catalog.sql

BEGIN;

-- Clear existing data (reverse order of dependencies)
DELETE FROM catalog.variant_media;
DELETE FROM catalog.product_media;
DELETE FROM catalog.variant_attributes;
DELETE FROM catalog.variants;
DELETE FROM catalog.products;
DELETE FROM catalog.categories;
DELETE FROM media.media_assets WHERE key LIKE 'seed/%';

-- ═══════════════════════════════════════════════════════════════════════════════
-- CATEGORIES
-- ═══════════════════════════════════════════════════════════════════════════════

INSERT INTO catalog.categories (id, name, slug, description, parent_id) VALUES
-- Parent categories
('11111111-0000-0000-0000-000000000001', 'Electronics', 'electronics', 'Electronic devices and gadgets', NULL),
('11111111-0000-0000-0000-000000000002', 'Fashion', 'fashion', 'Clothing and accessories', NULL),
('11111111-0000-0000-0000-000000000003', 'Home & Living', 'home-living', 'Home decor and furniture', NULL),

-- Electronics subcategories
('11111111-0000-0000-0001-000000000001', 'Smartphones', 'smartphones', 'Mobile phones', '11111111-0000-0000-0000-000000000001'),
('11111111-0000-0000-0001-000000000002', 'Laptops', 'laptops', 'Portable computers', '11111111-0000-0000-0000-000000000001'),
('11111111-0000-0000-0001-000000000003', 'Audio', 'audio', 'Headphones, speakers, earbuds', '11111111-0000-0000-0000-000000000001'),

-- Fashion subcategories
('11111111-0000-0000-0002-000000000001', 'Men', 'men', 'Men''s clothing', '11111111-0000-0000-0000-000000000002'),
('11111111-0000-0000-0002-000000000002', 'Women', 'women', 'Women''s clothing', '11111111-0000-0000-0000-000000000002'),
('11111111-0000-0000-0002-000000000003', 'Shoes', 'shoes', 'Footwear', '11111111-0000-0000-0000-000000000002'),

-- Home subcategories
('11111111-0000-0000-0003-000000000001', 'Furniture', 'furniture', 'Tables, chairs, storage', '11111111-0000-0000-0000-000000000003'),
('11111111-0000-0000-0003-000000000002', 'Decor', 'decor', 'Decorative items', '11111111-0000-0000-0000-000000000003');

-- ═══════════════════════════════════════════════════════════════════════════════
-- MEDIA ASSETS (Picsum placeholder images)
-- Using different Picsum IDs for variety
-- ═══════════════════════════════════════════════════════════════════════════════

INSERT INTO media.media_assets (id, key, bucket, content_type, size_bytes, cdn_url) VALUES
-- iPhone images
('22222222-0001-0000-0000-000000000001', 'seed/iphone-1.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/1/800/800'),
('22222222-0001-0000-0000-000000000002', 'seed/iphone-2.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/2/800/800'),
('22222222-0001-0000-0000-000000000003', 'seed/iphone-3.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/3/800/800'),

-- Samsung images
('22222222-0002-0000-0000-000000000001', 'seed/samsung-1.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/4/800/800'),
('22222222-0002-0000-0000-000000000002', 'seed/samsung-2.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/5/800/800'),

-- MacBook images
('22222222-0003-0000-0000-000000000001', 'seed/macbook-1.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/6/800/800'),
('22222222-0003-0000-0000-000000000002', 'seed/macbook-2.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/7/800/800'),
('22222222-0003-0000-0000-000000000003', 'seed/macbook-3.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/8/800/800'),
('22222222-0003-0000-0000-000000000004', 'seed/macbook-4.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/9/800/800'),

-- Earbuds images
('22222222-0004-0000-0000-000000000001', 'seed/earbuds-1.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/10/800/800'),
('22222222-0004-0000-0000-000000000002', 'seed/earbuds-2.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/11/800/800'),

-- T-Shirt images
('22222222-0005-0000-0000-000000000001', 'seed/tshirt-1.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/12/800/800'),
('22222222-0005-0000-0000-000000000002', 'seed/tshirt-2.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/13/800/800'),
('22222222-0005-0000-0000-000000000003', 'seed/tshirt-3.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/14/800/800'),

-- Dress images
('22222222-0006-0000-0000-000000000001', 'seed/dress-1.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/15/800/800'),
('22222222-0006-0000-0000-000000000002', 'seed/dress-2.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/16/800/800'),

-- Sneakers images
('22222222-0007-0000-0000-000000000001', 'seed/sneakers-1.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/17/800/800'),
('22222222-0007-0000-0000-000000000002', 'seed/sneakers-2.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/18/800/800'),
('22222222-0007-0000-0000-000000000003', 'seed/sneakers-3.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/19/800/800'),

-- Chair images
('22222222-0008-0000-0000-000000000001', 'seed/chair-1.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/20/800/800'),
('22222222-0008-0000-0000-000000000002', 'seed/chair-2.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/21/800/800'),

-- Watch images
('22222222-0009-0000-0000-000000000001', 'seed/watch-1.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/22/800/800'),
('22222222-0009-0000-0000-000000000002', 'seed/watch-2.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/23/800/800'),

-- Backpack images
('22222222-0010-0000-0000-000000000001', 'seed/backpack-1.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/24/800/800'),
('22222222-0010-0000-0000-000000000002', 'seed/backpack-2.jpg', 'products', 'image/jpeg', 50000, 'https://picsum.photos/id/25/800/800');

-- ═══════════════════════════════════════════════════════════════════════════════
-- PRODUCTS
-- ═══════════════════════════════════════════════════════════════════════════════

INSERT INTO catalog.products (id, category_id, name, slug, description, base_price_idr, weight_grams, sku, length_mm, width_mm, height_mm, popularity_score) VALUES

-- 1. iPhone 15 Pro
('33333333-0001-0000-0000-000000000001', 
 '11111111-0000-0000-0001-000000000001',
 'iPhone 15 Pro',
 'iphone-15-pro',
 'Latest iPhone with A17 Pro chip, titanium design, and advanced camera system. Features USB-C, Action button, and ProRes video recording.',
 18999000, 187, 'APPLE-IP15PRO', 147, 71, 8, 250),

-- 2. Samsung Galaxy S24 Ultra
('33333333-0002-0000-0000-000000000001',
 '11111111-0000-0000-0001-000000000001',
 'Samsung Galaxy S24 Ultra',
 'samsung-galaxy-s24-ultra',
 'Flagship Android with S Pen, 200MP camera, Galaxy AI features. Titanium frame and anti-reflective display.',
 19999000, 232, 'SAMSUNG-S24U', 162, 79, 9, 180),

-- 3. MacBook Pro 14" M3
('33333333-0003-0000-0000-000000000001',
 '11111111-0000-0000-0001-000000000002',
 'MacBook Pro 14-inch M3',
 'macbook-pro-14-m3',
 'Professional laptop with M3 chip, Liquid Retina XDR display, up to 22 hours battery. Perfect for creative professionals.',
 29999000, 1600, 'APPLE-MBP14M3', 312, 221, 16, 150),

-- 4. Pro Wireless Earbuds
('33333333-0004-0000-0000-000000000001',
 '11111111-0000-0000-0001-000000000003',
 'Pro Wireless Earbuds',
 'pro-wireless-earbuds',
 'Active noise cancellation, 30-hour battery life, premium sound quality. IPX4 water resistant with spatial audio.',
 1499000, 50, 'EARBUDS-PRO', 60, 50, 30, 320),

-- 5. Premium Cotton T-Shirt
('33333333-0005-0000-0000-000000000001',
 '11111111-0000-0000-0002-000000000001',
 'Premium Cotton T-Shirt',
 'premium-cotton-tshirt',
 '100% organic cotton, comfortable fit, pre-shrunk fabric. Perfect for everyday wear with minimal environmental impact.',
 199000, 200, 'TSHIRT-PREMIUM', 300, 200, 20, 500),

-- 6. Floral Summer Dress
('33333333-0006-0000-0000-000000000001',
 '11111111-0000-0000-0002-000000000002',
 'Floral Summer Dress',
 'floral-summer-dress',
 'Light and breezy dress with beautiful floral pattern. Midi length, adjustable straps, perfect for summer occasions.',
 399000, 250, 'DRESS-FLORAL', 350, 250, 30, 280),

-- 7. Urban Runner Sneakers
('33333333-0007-0000-0000-000000000001',
 '11111111-0000-0000-0002-000000000003',
 'Urban Runner Sneakers',
 'urban-runner-sneakers',
 'Lightweight running shoes with responsive cushioning. Breathable mesh upper, durable rubber outsole. Great for daily runs.',
 899000, 350, 'SNEAKERS-URBAN', 320, 110, 120, 420),

-- 8. Ergonomic Office Chair
('33333333-0008-0000-0000-000000000001',
 '11111111-0000-0000-0003-000000000001',
 'Ergonomic Office Chair',
 'ergonomic-office-chair',
 'Premium office chair with lumbar support, adjustable armrests, and breathable mesh back. Perfect for long work hours.',
 2499000, 15000, 'CHAIR-ERGO', 700, 700, 1200, 120),

-- 9. Smart Watch Pro
('33333333-0009-0000-0000-000000000001',
 '11111111-0000-0000-0001-000000000003',
 'Smart Watch Pro',
 'smart-watch-pro',
 'Advanced fitness tracking, heart rate monitor, GPS, always-on display. 7-day battery life with 50m water resistance.',
 2999000, 45, 'WATCH-SMART', 44, 38, 11, 350),

-- 10. Travel Backpack
('33333333-0010-0000-0000-000000000001',
 '11111111-0000-0000-0002-000000000001',
 'Travel Backpack 40L',
 'travel-backpack-40l',
 'Durable travel backpack with laptop compartment, anti-theft pocket, and compression straps. Carry-on compliant.',
 599000, 900, 'BACKPACK-40L', 550, 350, 200, 200);

-- ═══════════════════════════════════════════════════════════════════════════════
-- PRODUCT MEDIA (link products to images)
-- ═══════════════════════════════════════════════════════════════════════════════

INSERT INTO catalog.product_media (product_id, media_id, position, is_primary, alt_text) VALUES
-- iPhone
('33333333-0001-0000-0000-000000000001', '22222222-0001-0000-0000-000000000001', 0, true, 'iPhone 15 Pro - Front view'),
('33333333-0001-0000-0000-000000000001', '22222222-0001-0000-0000-000000000002', 1, false, 'iPhone 15 Pro - Back view'),
('33333333-0001-0000-0000-000000000001', '22222222-0001-0000-0000-000000000003', 2, false, 'iPhone 15 Pro - Side view'),

-- Samsung
('33333333-0002-0000-0000-000000000001', '22222222-0002-0000-0000-000000000001', 0, true, 'Samsung Galaxy S24 Ultra - Front'),
('33333333-0002-0000-0000-000000000001', '22222222-0002-0000-0000-000000000002', 1, false, 'Samsung Galaxy S24 Ultra - Back'),

-- MacBook
('33333333-0003-0000-0000-000000000001', '22222222-0003-0000-0000-000000000001', 0, true, 'MacBook Pro 14 - Open'),
('33333333-0003-0000-0000-000000000001', '22222222-0003-0000-0000-000000000002', 1, false, 'MacBook Pro 14 - Side'),
('33333333-0003-0000-0000-000000000001', '22222222-0003-0000-0000-000000000003', 2, false, 'MacBook Pro 14 - Keyboard'),
('33333333-0003-0000-0000-000000000001', '22222222-0003-0000-0000-000000000004', 3, false, 'MacBook Pro 14 - Ports'),

-- Earbuds
('33333333-0004-0000-0000-000000000001', '22222222-0004-0000-0000-000000000001', 0, true, 'Pro Wireless Earbuds - Case'),
('33333333-0004-0000-0000-000000000001', '22222222-0004-0000-0000-000000000002', 1, false, 'Pro Wireless Earbuds - In ear'),

-- T-Shirt
('33333333-0005-0000-0000-000000000001', '22222222-0005-0000-0000-000000000001', 0, true, 'Premium T-Shirt - Front'),
('33333333-0005-0000-0000-000000000001', '22222222-0005-0000-0000-000000000002', 1, false, 'Premium T-Shirt - Back'),
('33333333-0005-0000-0000-000000000001', '22222222-0005-0000-0000-000000000003', 2, false, 'Premium T-Shirt - Detail'),

-- Dress
('33333333-0006-0000-0000-000000000001', '22222222-0006-0000-0000-000000000001', 0, true, 'Floral Dress - Full'),
('33333333-0006-0000-0000-000000000001', '22222222-0006-0000-0000-000000000002', 1, false, 'Floral Dress - Detail'),

-- Sneakers
('33333333-0007-0000-0000-000000000001', '22222222-0007-0000-0000-000000000001', 0, true, 'Urban Runner - Side'),
('33333333-0007-0000-0000-000000000001', '22222222-0007-0000-0000-000000000002', 1, false, 'Urban Runner - Top'),
('33333333-0007-0000-0000-000000000001', '22222222-0007-0000-0000-000000000003', 2, false, 'Urban Runner - Sole'),

-- Chair
('33333333-0008-0000-0000-000000000001', '22222222-0008-0000-0000-000000000001', 0, true, 'Office Chair - Front'),
('33333333-0008-0000-0000-000000000001', '22222222-0008-0000-0000-000000000002', 1, false, 'Office Chair - Side'),

-- Watch
('33333333-0009-0000-0000-000000000001', '22222222-0009-0000-0000-000000000001', 0, true, 'Smart Watch - Face'),
('33333333-0009-0000-0000-000000000001', '22222222-0009-0000-0000-000000000002', 1, false, 'Smart Watch - Band'),

-- Backpack
('33333333-0010-0000-0000-000000000001', '22222222-0010-0000-0000-000000000001', 0, true, 'Backpack - Front'),
('33333333-0010-0000-0000-000000000001', '22222222-0010-0000-0000-000000000002', 1, false, 'Backpack - Open');

-- ═══════════════════════════════════════════════════════════════════════════════
-- VARIANTS (products with multiple options)
-- ═══════════════════════════════════════════════════════════════════════════════

-- iPhone 15 Pro variants (3 storage × 4 colors = 12 variants, showing 6)
INSERT INTO catalog.variants (id, product_id, name, sku, price_idr, stock, weight_grams) VALUES
('44444444-0001-0001-0000-000000000001', '33333333-0001-0000-0000-000000000001', '128GB Natural Titanium', 'IP15P-128-NAT', 18999000, 50, 187),
('44444444-0001-0002-0000-000000000001', '33333333-0001-0000-0000-000000000001', '128GB Blue Titanium', 'IP15P-128-BLU', 18999000, 45, 187),
('44444444-0001-0003-0000-000000000001', '33333333-0001-0000-0000-000000000001', '256GB Natural Titanium', 'IP15P-256-NAT', 21999000, 35, 187),
('44444444-0001-0004-0000-000000000001', '33333333-0001-0000-0000-000000000001', '256GB Black Titanium', 'IP15P-256-BLK', 21999000, 30, 187),
('44444444-0001-0005-0000-000000000001', '33333333-0001-0000-0000-000000000001', '512GB White Titanium', 'IP15P-512-WHT', 25999000, 20, 187),
('44444444-0001-0006-0000-000000000001', '33333333-0001-0000-0000-000000000001', '1TB Black Titanium', 'IP15P-1TB-BLK', 29999000, 10, 187);

-- Samsung S24 Ultra variants
INSERT INTO catalog.variants (id, product_id, name, sku, price_idr, stock, weight_grams) VALUES
('44444444-0002-0001-0000-000000000001', '33333333-0002-0000-0000-000000000001', '256GB Titanium Gray', 'S24U-256-GRY', 19999000, 40, 232),
('44444444-0002-0002-0000-000000000001', '33333333-0002-0000-0000-000000000001', '256GB Titanium Violet', 'S24U-256-VIO', 19999000, 35, 232),
('44444444-0002-0003-0000-000000000001', '33333333-0002-0000-0000-000000000001', '512GB Titanium Black', 'S24U-512-BLK', 23999000, 25, 232),
('44444444-0002-0004-0000-000000000001', '33333333-0002-0000-0000-000000000001', '1TB Titanium Gray', 'S24U-1TB-GRY', 27999000, 15, 232);

-- MacBook Pro variants
INSERT INTO catalog.variants (id, product_id, name, sku, price_idr, stock, weight_grams) VALUES
('44444444-0003-0001-0000-000000000001', '33333333-0003-0000-0000-000000000001', 'M3 8GB/512GB Space Gray', 'MBP14-M3-8-512-SG', 29999000, 20, 1600),
('44444444-0003-0002-0000-000000000001', '33333333-0003-0000-0000-000000000001', 'M3 8GB/512GB Silver', 'MBP14-M3-8-512-SL', 29999000, 18, 1600),
('44444444-0003-0003-0000-000000000001', '33333333-0003-0000-0000-000000000001', 'M3 Pro 18GB/1TB Space Gray', 'MBP14-M3P-18-1TB-SG', 39999000, 12, 1600),
('44444444-0003-0004-0000-000000000001', '33333333-0003-0000-0000-000000000001', 'M3 Max 36GB/1TB Space Gray', 'MBP14-M3X-36-1TB-SG', 54999000, 8, 1600);

-- Earbuds variants
INSERT INTO catalog.variants (id, product_id, name, sku, price_idr, stock, weight_grams) VALUES
('44444444-0004-0001-0000-000000000001', '33333333-0004-0000-0000-000000000001', 'Matte Black', 'EARBUD-BLK', 1499000, 80, 50),
('44444444-0004-0002-0000-000000000001', '33333333-0004-0000-0000-000000000001', 'Pearl White', 'EARBUD-WHT', 1499000, 75, 50),
('44444444-0004-0003-0000-000000000001', '33333333-0004-0000-0000-000000000001', 'Forest Green', 'EARBUD-GRN', 1499000, 40, 50);

-- T-Shirt variants (4 sizes × 3 colors = 12)
INSERT INTO catalog.variants (id, product_id, name, sku, price_idr, stock, weight_grams) VALUES
('44444444-0005-0001-0000-000000000001', '33333333-0005-0000-0000-000000000001', 'S Black', 'TSHIRT-S-BLK', 199000, 100, 180),
('44444444-0005-0002-0000-000000000001', '33333333-0005-0000-0000-000000000001', 'S White', 'TSHIRT-S-WHT', 199000, 100, 180),
('44444444-0005-0003-0000-000000000001', '33333333-0005-0000-0000-000000000001', 'S Navy', 'TSHIRT-S-NAV', 199000, 80, 180),
('44444444-0005-0004-0000-000000000001', '33333333-0005-0000-0000-000000000001', 'M Black', 'TSHIRT-M-BLK', 199000, 120, 200),
('44444444-0005-0005-0000-000000000001', '33333333-0005-0000-0000-000000000001', 'M White', 'TSHIRT-M-WHT', 199000, 120, 200),
('44444444-0005-0006-0000-000000000001', '33333333-0005-0000-0000-000000000001', 'M Navy', 'TSHIRT-M-NAV', 199000, 90, 200),
('44444444-0005-0007-0000-000000000001', '33333333-0005-0000-0000-000000000001', 'L Black', 'TSHIRT-L-BLK', 199000, 100, 220),
('44444444-0005-0008-0000-000000000001', '33333333-0005-0000-0000-000000000001', 'L White', 'TSHIRT-L-WHT', 199000, 100, 220),
('44444444-0005-0009-0000-000000000001', '33333333-0005-0000-0000-000000000001', 'L Navy', 'TSHIRT-L-NAV', 199000, 75, 220),
('44444444-0005-0010-0000-000000000001', '33333333-0005-0000-0000-000000000001', 'XL Black', 'TSHIRT-XL-BLK', 199000, 80, 240),
('44444444-0005-0011-0000-000000000001', '33333333-0005-0000-0000-000000000001', 'XL White', 'TSHIRT-XL-WHT', 199000, 80, 240),
('44444444-0005-0012-0000-000000000001', '33333333-0005-0000-0000-000000000001', 'XL Navy', 'TSHIRT-XL-NAV', 199000, 60, 240);

-- Dress variants (3 sizes × 3 patterns = 9)
INSERT INTO catalog.variants (id, product_id, name, sku, price_idr, stock, weight_grams) VALUES
('44444444-0006-0001-0000-000000000001', '33333333-0006-0000-0000-000000000001', 'S Blue Floral', 'DRESS-S-BLU', 399000, 50, 240),
('44444444-0006-0002-0000-000000000001', '33333333-0006-0000-0000-000000000001', 'S Pink Floral', 'DRESS-S-PNK', 399000, 45, 240),
('44444444-0006-0003-0000-000000000001', '33333333-0006-0000-0000-000000000001', 'S Yellow Floral', 'DRESS-S-YLW', 399000, 40, 240),
('44444444-0006-0004-0000-000000000001', '33333333-0006-0000-0000-000000000001', 'M Blue Floral', 'DRESS-M-BLU', 399000, 60, 250),
('44444444-0006-0005-0000-000000000001', '33333333-0006-0000-0000-000000000001', 'M Pink Floral', 'DRESS-M-PNK', 399000, 55, 250),
('44444444-0006-0006-0000-000000000001', '33333333-0006-0000-0000-000000000001', 'M Yellow Floral', 'DRESS-M-YLW', 399000, 45, 250),
('44444444-0006-0007-0000-000000000001', '33333333-0006-0000-0000-000000000001', 'L Blue Floral', 'DRESS-L-BLU', 399000, 40, 260),
('44444444-0006-0008-0000-000000000001', '33333333-0006-0000-0000-000000000001', 'L Pink Floral', 'DRESS-L-PNK', 399000, 35, 260),
('44444444-0006-0009-0000-000000000001', '33333333-0006-0000-0000-000000000001', 'L Yellow Floral', 'DRESS-L-YLW', 399000, 30, 260);

-- Sneakers variants (5 sizes × 3 colors = 15, showing 9)
INSERT INTO catalog.variants (id, product_id, name, sku, price_idr, stock, weight_grams) VALUES
('44444444-0007-0001-0000-000000000001', '33333333-0007-0000-0000-000000000001', '40 Black/White', 'SNKR-40-BW', 899000, 30, 340),
('44444444-0007-0002-0000-000000000001', '33333333-0007-0000-0000-000000000001', '41 Black/White', 'SNKR-41-BW', 899000, 35, 350),
('44444444-0007-0003-0000-000000000001', '33333333-0007-0000-0000-000000000001', '42 Black/White', 'SNKR-42-BW', 899000, 40, 360),
('44444444-0007-0004-0000-000000000001', '33333333-0007-0000-0000-000000000001', '42 Navy/Gray', 'SNKR-42-NG', 899000, 35, 360),
('44444444-0007-0005-0000-000000000001', '33333333-0007-0000-0000-000000000001', '43 Black/White', 'SNKR-43-BW', 899000, 45, 370),
('44444444-0007-0006-0000-000000000001', '33333333-0007-0000-0000-000000000001', '43 Navy/Gray', 'SNKR-43-NG', 899000, 40, 370),
('44444444-0007-0007-0000-000000000001', '33333333-0007-0000-0000-000000000001', '43 All White', 'SNKR-43-AW', 899000, 30, 370),
('44444444-0007-0008-0000-000000000001', '33333333-0007-0000-0000-000000000001', '44 Black/White', 'SNKR-44-BW', 899000, 35, 380),
('44444444-0007-0009-0000-000000000001', '33333333-0007-0000-0000-000000000001', '45 Black/White', 'SNKR-45-BW', 899000, 25, 390);

-- Chair variants
INSERT INTO catalog.variants (id, product_id, name, sku, price_idr, stock, weight_grams) VALUES
('44444444-0008-0001-0000-000000000001', '33333333-0008-0000-0000-000000000001', 'Black Mesh', 'CHAIR-BLK', 2499000, 25, 15000),
('44444444-0008-0002-0000-000000000001', '33333333-0008-0000-0000-000000000001', 'Gray Mesh', 'CHAIR-GRY', 2499000, 20, 15000),
('44444444-0008-0003-0000-000000000001', '33333333-0008-0000-0000-000000000001', 'White Mesh', 'CHAIR-WHT', 2599000, 15, 15000);

-- Watch variants
INSERT INTO catalog.variants (id, product_id, name, sku, price_idr, stock, weight_grams) VALUES
('44444444-0009-0001-0000-000000000001', '33333333-0009-0000-0000-000000000001', '41mm Midnight', 'WATCH-41-MID', 2999000, 50, 42),
('44444444-0009-0002-0000-000000000001', '33333333-0009-0000-0000-000000000001', '41mm Starlight', 'WATCH-41-STR', 2999000, 45, 42),
('44444444-0009-0003-0000-000000000001', '33333333-0009-0000-0000-000000000001', '45mm Midnight', 'WATCH-45-MID', 3299000, 40, 48),
('44444444-0009-0004-0000-000000000001', '33333333-0009-0000-0000-000000000001', '45mm Starlight', 'WATCH-45-STR', 3299000, 35, 48);

-- Backpack variants
INSERT INTO catalog.variants (id, product_id, name, sku, price_idr, stock, weight_grams) VALUES
('44444444-0010-0001-0000-000000000001', '33333333-0010-0000-0000-000000000001', 'Black', 'BKPK-BLK', 599000, 60, 900),
('44444444-0010-0002-0000-000000000001', '33333333-0010-0000-0000-000000000001', 'Navy', 'BKPK-NAV', 599000, 50, 900),
('44444444-0010-0003-0000-000000000001', '33333333-0010-0000-0000-000000000001', 'Olive', 'BKPK-OLV', 599000, 40, 900);

-- ═══════════════════════════════════════════════════════════════════════════════
-- VARIANT ATTRIBUTES
-- ═══════════════════════════════════════════════════════════════════════════════

-- iPhone attributes
INSERT INTO catalog.variant_attributes (variant_id, key, value) VALUES
('44444444-0001-0001-0000-000000000001', 'Storage', '128GB'),
('44444444-0001-0001-0000-000000000001', 'Color', 'Natural Titanium'),
('44444444-0001-0002-0000-000000000001', 'Storage', '128GB'),
('44444444-0001-0002-0000-000000000001', 'Color', 'Blue Titanium'),
('44444444-0001-0003-0000-000000000001', 'Storage', '256GB'),
('44444444-0001-0003-0000-000000000001', 'Color', 'Natural Titanium'),
('44444444-0001-0004-0000-000000000001', 'Storage', '256GB'),
('44444444-0001-0004-0000-000000000001', 'Color', 'Black Titanium'),
('44444444-0001-0005-0000-000000000001', 'Storage', '512GB'),
('44444444-0001-0005-0000-000000000001', 'Color', 'White Titanium'),
('44444444-0001-0006-0000-000000000001', 'Storage', '1TB'),
('44444444-0001-0006-0000-000000000001', 'Color', 'Black Titanium');

-- Samsung attributes
INSERT INTO catalog.variant_attributes (variant_id, key, value) VALUES
('44444444-0002-0001-0000-000000000001', 'Storage', '256GB'),
('44444444-0002-0001-0000-000000000001', 'Color', 'Titanium Gray'),
('44444444-0002-0002-0000-000000000001', 'Storage', '256GB'),
('44444444-0002-0002-0000-000000000001', 'Color', 'Titanium Violet'),
('44444444-0002-0003-0000-000000000001', 'Storage', '512GB'),
('44444444-0002-0003-0000-000000000001', 'Color', 'Titanium Black'),
('44444444-0002-0004-0000-000000000001', 'Storage', '1TB'),
('44444444-0002-0004-0000-000000000001', 'Color', 'Titanium Gray');

-- MacBook attributes
INSERT INTO catalog.variant_attributes (variant_id, key, value) VALUES
('44444444-0003-0001-0000-000000000001', 'Chip', 'M3'),
('44444444-0003-0001-0000-000000000001', 'RAM', '8GB'),
('44444444-0003-0001-0000-000000000001', 'Storage', '512GB'),
('44444444-0003-0001-0000-000000000001', 'Color', 'Space Gray'),
('44444444-0003-0002-0000-000000000001', 'Chip', 'M3'),
('44444444-0003-0002-0000-000000000001', 'RAM', '8GB'),
('44444444-0003-0002-0000-000000000001', 'Storage', '512GB'),
('44444444-0003-0002-0000-000000000001', 'Color', 'Silver'),
('44444444-0003-0003-0000-000000000001', 'Chip', 'M3 Pro'),
('44444444-0003-0003-0000-000000000001', 'RAM', '18GB'),
('44444444-0003-0003-0000-000000000001', 'Storage', '1TB'),
('44444444-0003-0003-0000-000000000001', 'Color', 'Space Gray'),
('44444444-0003-0004-0000-000000000001', 'Chip', 'M3 Max'),
('44444444-0003-0004-0000-000000000001', 'RAM', '36GB'),
('44444444-0003-0004-0000-000000000001', 'Storage', '1TB'),
('44444444-0003-0004-0000-000000000001', 'Color', 'Space Gray');

-- Earbuds attributes
INSERT INTO catalog.variant_attributes (variant_id, key, value) VALUES
('44444444-0004-0001-0000-000000000001', 'Color', 'Matte Black'),
('44444444-0004-0002-0000-000000000001', 'Color', 'Pearl White'),
('44444444-0004-0003-0000-000000000001', 'Color', 'Forest Green');

-- T-Shirt attributes
INSERT INTO catalog.variant_attributes (variant_id, key, value) VALUES
('44444444-0005-0001-0000-000000000001', 'Size', 'S'), ('44444444-0005-0001-0000-000000000001', 'Color', 'Black'),
('44444444-0005-0002-0000-000000000001', 'Size', 'S'), ('44444444-0005-0002-0000-000000000001', 'Color', 'White'),
('44444444-0005-0003-0000-000000000001', 'Size', 'S'), ('44444444-0005-0003-0000-000000000001', 'Color', 'Navy'),
('44444444-0005-0004-0000-000000000001', 'Size', 'M'), ('44444444-0005-0004-0000-000000000001', 'Color', 'Black'),
('44444444-0005-0005-0000-000000000001', 'Size', 'M'), ('44444444-0005-0005-0000-000000000001', 'Color', 'White'),
('44444444-0005-0006-0000-000000000001', 'Size', 'M'), ('44444444-0005-0006-0000-000000000001', 'Color', 'Navy'),
('44444444-0005-0007-0000-000000000001', 'Size', 'L'), ('44444444-0005-0007-0000-000000000001', 'Color', 'Black'),
('44444444-0005-0008-0000-000000000001', 'Size', 'L'), ('44444444-0005-0008-0000-000000000001', 'Color', 'White'),
('44444444-0005-0009-0000-000000000001', 'Size', 'L'), ('44444444-0005-0009-0000-000000000001', 'Color', 'Navy'),
('44444444-0005-0010-0000-000000000001', 'Size', 'XL'), ('44444444-0005-0010-0000-000000000001', 'Color', 'Black'),
('44444444-0005-0011-0000-000000000001', 'Size', 'XL'), ('44444444-0005-0011-0000-000000000001', 'Color', 'White'),
('44444444-0005-0012-0000-000000000001', 'Size', 'XL'), ('44444444-0005-0012-0000-000000000001', 'Color', 'Navy');

-- Dress attributes
INSERT INTO catalog.variant_attributes (variant_id, key, value) VALUES
('44444444-0006-0001-0000-000000000001', 'Size', 'S'), ('44444444-0006-0001-0000-000000000001', 'Pattern', 'Blue Floral'),
('44444444-0006-0002-0000-000000000001', 'Size', 'S'), ('44444444-0006-0002-0000-000000000001', 'Pattern', 'Pink Floral'),
('44444444-0006-0003-0000-000000000001', 'Size', 'S'), ('44444444-0006-0003-0000-000000000001', 'Pattern', 'Yellow Floral'),
('44444444-0006-0004-0000-000000000001', 'Size', 'M'), ('44444444-0006-0004-0000-000000000001', 'Pattern', 'Blue Floral'),
('44444444-0006-0005-0000-000000000001', 'Size', 'M'), ('44444444-0006-0005-0000-000000000001', 'Pattern', 'Pink Floral'),
('44444444-0006-0006-0000-000000000001', 'Size', 'M'), ('44444444-0006-0006-0000-000000000001', 'Pattern', 'Yellow Floral'),
('44444444-0006-0007-0000-000000000001', 'Size', 'L'), ('44444444-0006-0007-0000-000000000001', 'Pattern', 'Blue Floral'),
('44444444-0006-0008-0000-000000000001', 'Size', 'L'), ('44444444-0006-0008-0000-000000000001', 'Pattern', 'Pink Floral'),
('44444444-0006-0009-0000-000000000001', 'Size', 'L'), ('44444444-0006-0009-0000-000000000001', 'Pattern', 'Yellow Floral');

-- Sneakers attributes
INSERT INTO catalog.variant_attributes (variant_id, key, value) VALUES
('44444444-0007-0001-0000-000000000001', 'Size', '40'), ('44444444-0007-0001-0000-000000000001', 'Color', 'Black/White'),
('44444444-0007-0002-0000-000000000001', 'Size', '41'), ('44444444-0007-0002-0000-000000000001', 'Color', 'Black/White'),
('44444444-0007-0003-0000-000000000001', 'Size', '42'), ('44444444-0007-0003-0000-000000000001', 'Color', 'Black/White'),
('44444444-0007-0004-0000-000000000001', 'Size', '42'), ('44444444-0007-0004-0000-000000000001', 'Color', 'Navy/Gray'),
('44444444-0007-0005-0000-000000000001', 'Size', '43'), ('44444444-0007-0005-0000-000000000001', 'Color', 'Black/White'),
('44444444-0007-0006-0000-000000000001', 'Size', '43'), ('44444444-0007-0006-0000-000000000001', 'Color', 'Navy/Gray'),
('44444444-0007-0007-0000-000000000001', 'Size', '43'), ('44444444-0007-0007-0000-000000000001', 'Color', 'All White'),
('44444444-0007-0008-0000-000000000001', 'Size', '44'), ('44444444-0007-0008-0000-000000000001', 'Color', 'Black/White'),
('44444444-0007-0009-0000-000000000001', 'Size', '45'), ('44444444-0007-0009-0000-000000000001', 'Color', 'Black/White');

-- Chair attributes
INSERT INTO catalog.variant_attributes (variant_id, key, value) VALUES
('44444444-0008-0001-0000-000000000001', 'Color', 'Black'),
('44444444-0008-0001-0000-000000000001', 'Material', 'Mesh'),
('44444444-0008-0002-0000-000000000001', 'Color', 'Gray'),
('44444444-0008-0002-0000-000000000001', 'Material', 'Mesh'),
('44444444-0008-0003-0000-000000000001', 'Color', 'White'),
('44444444-0008-0003-0000-000000000001', 'Material', 'Mesh');

-- Watch attributes
INSERT INTO catalog.variant_attributes (variant_id, key, value) VALUES
('44444444-0009-0001-0000-000000000001', 'Size', '41mm'), ('44444444-0009-0001-0000-000000000001', 'Color', 'Midnight'),
('44444444-0009-0002-0000-000000000001', 'Size', '41mm'), ('44444444-0009-0002-0000-000000000001', 'Color', 'Starlight'),
('44444444-0009-0003-0000-000000000001', 'Size', '45mm'), ('44444444-0009-0003-0000-000000000001', 'Color', 'Midnight'),
('44444444-0009-0004-0000-000000000001', 'Size', '45mm'), ('44444444-0009-0004-0000-000000000001', 'Color', 'Starlight');

-- Backpack attributes
INSERT INTO catalog.variant_attributes (variant_id, key, value) VALUES
('44444444-0010-0001-0000-000000000001', 'Color', 'Black'),
('44444444-0010-0002-0000-000000000001', 'Color', 'Navy'),
('44444444-0010-0003-0000-000000000001', 'Color', 'Olive');

COMMIT;

-- Summary
SELECT 'Categories' as entity, COUNT(*) as count FROM catalog.categories
UNION ALL
SELECT 'Products', COUNT(*) FROM catalog.products
UNION ALL
SELECT 'Variants', COUNT(*) FROM catalog.variants
UNION ALL
SELECT 'Variant Attributes', COUNT(*) FROM catalog.variant_attributes
UNION ALL
SELECT 'Media Assets', COUNT(*) FROM media.media_assets WHERE key LIKE 'seed/%'
UNION ALL
SELECT 'Product Media Links', COUNT(*) FROM catalog.product_media;
